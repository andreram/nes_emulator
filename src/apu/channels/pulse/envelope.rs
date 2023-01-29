bitflags! {
  pub struct PulseEnvelope: u8 {
    const VOLUME              = 0b00001111;
    const CONSTANT_VOLUME     = 0b00010000;
    const LENGTH_COUNTER_HALT = 0b00100000;
    const DUTY_2              = 0b01000000;
    const DUTY_1              = 0b10000000;
  }
}

impl PulseEnvelope {
  pub fn new() -> Self {
    PulseEnvelope::from_bits_truncate(0)
  }

  pub fn update(&mut self, data: u8) {
    self.bits = data;
  }

  pub fn get_envelope_volume(&self) -> u8 {
    self.bits & PulseEnvelope::VOLUME.bits
  }

  pub fn get_duty_cycle(&self) -> u8 {
    (self.bits & (PulseEnvelope::DUTY_1 | PulseEnvelope::DUTY_2).bits) >> 6
  }
}
