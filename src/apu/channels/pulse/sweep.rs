bitflags! {
  pub struct PulseSweep: u8 {
    const SHIFT   = 0b00000111;
    const NEGATE  = 0b00001000;
    const PERIOD  = 0b01110000;
    const ENABLED = 0b10000000;
  }
}

impl PulseSweep {
  pub fn new() -> Self {
    PulseSweep::from_bits_truncate(0)
  }

  pub fn update(&mut self, data: u8) {
    self.bits = data;
  }
}
