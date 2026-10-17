use embassy_rp::{
    bind_interrupts,
    i2c::{Async, Config, Error, I2c},
    peripherals::{I2C0, PIN_4, PIN_5},
    Peri,
};
use embassy_time::{Duration, Ticker};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
});

const ADDRESS: u8 = 0x68;
const PWR_MGMT_1: u8 = 0x6b;
const CONFIG: u8 = 0x1a;
const ACCEL_XOUT_H: u8 = 0x3b;

const GYRO_COUNTS_PER_DPS: f32 = 65.5;
const ACC_COUNTS_PER_G: f32 = 4096.0;

pub struct GyroAndAcc {
    pub acc_x: f32,
    pub acc_y: f32,
    pub acc_z: f32,

    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}

pub struct Gyro<'d> {
    i2c: I2c<'d, I2C0, Async>,
    calibration: GyroAndAcc,
}

impl<'d> Gyro<'d> {
    pub fn new(i2c: Peri<'d, I2C0>, scl: Peri<'d, PIN_5>, sda: Peri<'d, PIN_4>) -> Self {
        let mut config = Config::default();
        config.frequency = 400_000;

        Self {
            i2c: I2c::new_async(i2c, scl, sda, Irqs, config),
            calibration: GyroAndAcc {
                acc_x: 0.,
                acc_y: 0.,
                acc_z: 0.,

                gyro_x: 0.,
                gyro_y: 0.,
                gyro_z: 0.,
            },
        }
    }

    pub async fn calibrate(&mut self) -> Result<(), Error> {
        let mut loop_timer = Ticker::every(Duration::from_micros(1_000));

        for _ in 0..2000 {
            let data = self.read_gyro_and_acc_uncalibrated().await?;

            self.calibration.acc_x += data.acc_x;
            self.calibration.acc_y += data.acc_y;
            self.calibration.acc_z += data.acc_z;

            self.calibration.gyro_x += data.gyro_x;
            self.calibration.gyro_y += data.gyro_y;
            self.calibration.gyro_z += data.gyro_z;

            loop_timer.next().await;
        }

        self.calibration.acc_x /= 2000.;
        self.calibration.acc_y /= 2000.;
        self.calibration.acc_z = (self.calibration.acc_z / 2000.) - 1.;

        self.calibration.gyro_x /= 2000.;
        self.calibration.gyro_y /= 2000.;
        self.calibration.gyro_z /= 2000.;

        Ok(())
    }

    pub async fn read_gyro_and_acc(&mut self) -> Result<GyroAndAcc, Error> {
        let mut data = self.read_gyro_and_acc_uncalibrated().await?;

        data.acc_x -= self.calibration.acc_x;
        data.acc_y -= self.calibration.acc_y;
        data.acc_z -= self.calibration.acc_z;

        data.gyro_x -= self.calibration.gyro_x;
        data.gyro_y -= self.calibration.gyro_y;
        data.gyro_z -= self.calibration.gyro_z;

        Ok(data)
    }

    async fn read_gyro_and_acc_uncalibrated(&mut self) -> Result<GyroAndAcc, Error> {
        self.i2c.write_async(ADDRESS, [PWR_MGMT_1, 0x00]).await?;

        // DLPF 0x05, gyro +/-500 deg/s, accelerometer +/-8 g.
        self.i2c
            .write_async(ADDRESS, [CONFIG, 0x05, 0x08, 0x10])
            .await?;

        // One burst contains accelerometer, temperature, and gyroscope registers.
        let mut bytes = [0; 14];
        self.i2c
            .write_read_async(ADDRESS, [ACCEL_XOUT_H], &mut bytes)
            .await?;

        Ok(GyroAndAcc {
            acc_x: i16::from_be_bytes([bytes[0], bytes[1]]) as f32 / ACC_COUNTS_PER_G,
            acc_y: i16::from_be_bytes([bytes[2], bytes[3]]) as f32 / ACC_COUNTS_PER_G,
            acc_z: i16::from_be_bytes([bytes[4], bytes[5]]) as f32 / ACC_COUNTS_PER_G,

            gyro_x: i16::from_be_bytes([bytes[8], bytes[9]]) as f32 / GYRO_COUNTS_PER_DPS,
            gyro_y: i16::from_be_bytes([bytes[10], bytes[11]]) as f32 / GYRO_COUNTS_PER_DPS,
            gyro_z: i16::from_be_bytes([bytes[12], bytes[13]]) as f32 / GYRO_COUNTS_PER_DPS,
        })
    }
}
