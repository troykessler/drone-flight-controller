use embassy_rp::{Peri};
use embassy_rp::pwm::{ChannelAPin, Config, Pwm, SetDutyCycle, Slice};

const DESIRED_HZ_FREQUENCY: u32 = 400;
const DIVIDER: u8 = 16;

pub struct Motor<'a> {
    pwm: Pwm<'a>,
    throttle: u16,
}

impl<'a> Motor<'a> {
    pub fn new<T: Slice>(slice: Peri<'a, T>, pin: Peri<'a, impl ChannelAPin<T>>) -> Self {
        let period = embassy_rp::clocks::clk_sys_freq() / (DESIRED_HZ_FREQUENCY * DIVIDER as u32) - 1;

        let mut c = Config::default();
        c.top = period as u16;
        c.divider = DIVIDER.into();

        let min_duty = Self::pulse_to_ticks(1000);

        let mut pwm = Pwm::new_output_a(slice, pin, c.clone());
        pwm.set_duty_cycle(min_duty).unwrap();

        Self {
            pwm,
            throttle: 0,
        }
    }

    fn pulse_to_ticks(pulse_us: u32) -> u16 {
        (u64::from(embassy_rp::clocks::clk_sys_freq()) * u64::from(pulse_us) / (u64::from(DIVIDER) * 1_000_000)) as u16
    }

    pub fn get_throttle(&self) -> u16 {
        self.throttle
    }

    pub fn set_throttle(&mut self, throttle: u16) {
        let throttle = throttle.min(1023);

        let min_duty = Self::pulse_to_ticks(1000);
        let max_duty = Self::pulse_to_ticks(2000);

        let duty = (u32::from(min_duty) + u32::from(max_duty - min_duty) * u32::from(throttle) / 1023u32) as u16;

        self.pwm.set_duty_cycle(duty).unwrap();
        self.throttle = throttle;
    }
}
