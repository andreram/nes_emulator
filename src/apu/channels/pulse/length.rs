bitflags! {
  pub struct PulseLength: u8 {
    const TIMER_HIGH           = 0b00000111;
    const LENGTH_COUNTER_LOAD  = 0b11111000;
  }
}

impl PulseLength {
  pub fn new() -> Self {
    PulseLength::from_bits_truncate(0)
  }

  pub fn update(&mut self, data: u8) {
    self.bits = data;
  }

  pub fn read_length(&self) -> u8 {
    (self.bits & PulseLength::LENGTH_COUNTER_LOAD.bits) >> 3
  }

  pub fn read_timer_high(&self) -> u16 {
    ((self.bits & PulseLength::TIMER_HIGH.bits) as u16) << 8
  }
}
