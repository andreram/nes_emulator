pub struct PulseTimer {
  timer_low: u8,
}

impl PulseTimer {
  pub fn new() -> Self {
    PulseTimer { timer_low: 0 }
  }

  pub fn update(&mut self, data: u8) {
    self.timer_low = data;
  }

  pub fn read_timer_low(&self) -> u8 {
    self.timer_low
  }
}
