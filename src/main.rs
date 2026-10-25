#![no_std]
#![no_main]

// mod gyro;
mod motor;

use motor::Motor;

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::Driver,
};
use embassy_time::{Timer};
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
    // pin 4,5 -> gyro
    // pin 6,8,10,12 -> esc
    let p = embassy_rp::init(Default::default());
    spawner.spawn(serial_task(Driver::new(p.USB, Irqs)).unwrap());

    // https://pip-assets.raspberrypi.com/categories/814-rp2040/documents/RP-008371-DS-1-rp2040-datasheet.pdf#page=13
    let mut motor_1 = Motor::new(p.PWM_SLICE3, p.PIN_6);
    // let mut motor_2 = Motor::new(p.PWM_SLICE4, p.PIN_8);
    // let mut motor_3 = Motor::new(p.PWM_SLICE5, p.PIN_10);
    // let mut motor_4 = Motor::new(p.PWM_SLICE6, p.PIN_12);

    loop {
        for throttle in 0..1024 {
            motor_1.set_throttle(throttle);
            log::info!("throttle: {}", motor_1.get_throttle());
            Timer::after_millis(3).await;
        }
    }
}
