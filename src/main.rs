#![no_std]
#![no_main]

mod gyro;

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::Driver,
};
use embassy_time::{Duration, Ticker};
use gyro::Gyro;
use panic_halt as _;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn serial_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(serial_task(Driver::new(p.USB, Irqs)).unwrap());

    let mut gyro = Gyro::new(p.I2C0, p.PIN_5, p.PIN_4);

    let mut loop_timer = Ticker::every(Duration::from_micros(2_500));

    loop {
        if let Ok(data) = gyro.read_gyro_and_acc().await {
            log::info!(
                "gyro: X={:.1} Y={:.1} Z={:.1} deg/s, acc: X={:.2} Y={:.2} Z={:.2} g",
                data.gyro_x,
                data.gyro_y,
                data.gyro_z,
                data.acc_x,
                data.acc_y,
                data.acc_z
            );
        } else {
            log::warn!("GY-521 read failed");
        }

        loop_timer.next().await;
    }
}
