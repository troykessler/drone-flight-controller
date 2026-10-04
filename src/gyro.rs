use embassy_rp::{
    bind_interrupts,
    i2c::{Async, Config, Error, I2c},
    peripherals::{I2C0, PIN_4, PIN_5},
    Peri,
};

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
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub acc_x: f32,
    pub acc_y: f32,
    pub acc_z: f32,
}

pub struct Gyro<'d> {
    i2c: I2c<'d, I2C0, Async>,
}

impl<'d> Gyro<'d> {
    pub fn new(i2c: Peri<'d, I2C0>, scl: Peri<'d, PIN_5>, sda: Peri<'d, PIN_4>) -> Self {
        let mut config = Config::default();
        config.frequency = 400_000;

        Self {
            i2c: I2c::new_async(i2c, scl, sda, Irqs, config),
        }
    }

    pub async fn read_gyro_and_acc(&mut self) -> Result<GyroAndAcc, Error> {
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
            acc_x: raw_value(bytes[0], bytes[1]) / ACC_COUNTS_PER_G,
            acc_y: raw_value(bytes[2], bytes[3]) / ACC_COUNTS_PER_G,
            acc_z: raw_value(bytes[4], bytes[5]) / ACC_COUNTS_PER_G,
            gyro_x: raw_value(bytes[8], bytes[9]) / GYRO_COUNTS_PER_DPS,
            gyro_y: raw_value(bytes[10], bytes[11]) / GYRO_COUNTS_PER_DPS,
            gyro_z: raw_value(bytes[12], bytes[13]) / GYRO_COUNTS_PER_DPS,
        })
    }
}

fn raw_value(high: u8, low: u8) -> f32 {
    i16::from_be_bytes([high, low]) as f32
}
