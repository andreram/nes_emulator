bitflags! {
  pub struct APUStatus: u8 {
    const DMC_INTERRUPT   = 0b10000000;
    const FRAME_INTERRUPT = 0b01000000;
    const DMC_ACTIVE      = 0b00010000;
    const NOISE           = 0b00001000;
    const TRIANGLE        = 0b00000100;
    const PULSE_2         = 0b00000010;
    const PULSE_1         = 0b00000001;
  }
}

impl APUStatus {
  pub fn new() -> Self {
    APUStatus::from_bits_truncate(0)
  }

  pub fn update(&mut self, data: u8) {
    self.bits = data;
  }
}