#![no_std]
#![no_main]

// mod gyro;
mod motor;

use embassy_time::{Duration, Ticker};
use motor::Motor;

use uf_crsf::{CrsfParser, Packet};
use embassy_executor::Spawner;
use embassy_rp::{
    Peri, bind_interrupts, peripherals::{DMA_CH1, PIN_1, UART0, USB}, uart::{Config as UartConfig, UartRx}, usb::Driver
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use panic_halt as _;

type ChannelsType = Mutex<ThreadModeRawMutex, [u16; 16]>;
static CHANNELS: ChannelsType = Mutex::new([0u16; 16]);

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    UART0_IRQ => embassy_rp::uart::InterruptHandler<embassy_rp::peripherals::UART0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH1>;
});

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(spawner: Spawner) {
    // pin 4,5 -> gyro
    // pin 6,8,10,12 -> esc
    let p = embassy_rp::init(Default::default());

    spawner.spawn(serial_task(Driver::new(p.USB, Irqs)).unwrap());
    spawner.spawn(elrs_receive(p.UART0, p.PIN_1, p.DMA_CH1).unwrap());

    // https://pip-assets.raspberrypi.com/categories/814-rp2040/documents/RP-008371-DS-1-rp2040-datasheet.pdf#page=13
    let mut motor_1 = Motor::new(p.PWM_SLICE3, p.PIN_6);
    // let mut motor_2 = Motor::new(p.PWM_SLICE4, p.PIN_8);
    // let mut motor_3 = Motor::new(p.PWM_SLICE5, p.PIN_10);
    // let mut motor_4 = Motor::new(p.PWM_SLICE6, p.PIN_12);

    let mut ticker = Ticker::every(Duration::from_micros(2_500));

    loop {
        let throttle: u16;

        {
            let t = (*(CHANNELS.lock().await))[2];

            // https://github.com/tbs-fpv/tbs-crsf-spec/blob/main/crsf.md#0x16-rc-channels-packed-payload
            throttle = ((i32::from(t) - 992) * 5 / 8 + 1500).clamp(1000, 2000) as u16;
        }

        motor_1.set_throttle(throttle);
        log::info!("throttle: {}", motor_1.get_throttle());

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn serial_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn elrs_receive(
    uart0: Peri<'static, UART0>,
    pin1: Peri<'static, PIN_1>,
    dma1: Peri<'static, DMA_CH1>,
) {
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 420_000;

    let mut uart = UartRx::new(
        uart0,
        pin1,
        Irqs,
        dma1,
        uart_config,
    );

    let mut parser = CrsfParser::new();
    let mut rx_buf = [0u8; 64];

    loop {
        match uart.read(&mut rx_buf).await {
            Ok(()) => {
                let mut remaining: &[u8] = &rx_buf;

                for result in parser.iter_packets(&mut remaining) {
                    match result {
                        Ok(Packet::RCChannels(channels)) => {
                            {
                                *(CHANNELS.lock().await) = channels.0;
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    }
}
