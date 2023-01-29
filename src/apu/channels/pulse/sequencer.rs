pub struct PulseSequencer {
  divider: u16,
  step: u8,
}

impl PulseSequencer {
  pub fn new() -> Self {
    PulseSequencer {
      divider: 0,
      step: 0,
    }
  }

  pub fn update_divider(&mut self, data: u16) {
    self.divider = data;
  }

  pub fn reset_step(&mut self) {
    self.step = 0;
  }

  pub fn get_divider(&self) -> u16 {
    self.divider
  }

  pub fn get_sequence_step(&self) -> u16 {
    self.divider
  }

  pub fn decrement_divider(&mut self) {
    self.divider -= 1;
  }

  pub fn clock_sequence_step(&mut self) {
    self.step = (self.step + 1) % 8;
  }
}
