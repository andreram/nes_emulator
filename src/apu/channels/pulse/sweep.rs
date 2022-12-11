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

  // pub fn vram_addr_increment(&self) -> u8 {
  //   if self.contains(PulseEnvelope::VRAM_ADD_INCREMENT) {
  //     32
  //   } else {
  //     1
  //   }
  // }

  // pub fn should_generate_vblank_nmi(&self) -> bool {
  //   self.contains(PulseEnvelope::GENERATE_NMI)
  // }

  // pub fn background_pattern_table_addr(&self) -> u16 {
  //   if self.contains(PulseEnvelope::BACKROUND_PATTERN_ADDR) {
  //     0x1000
  //   } else {
  //     0
  //   }
  // }

  // pub fn sprite_pattern_table_addr(&self) -> u16 {
  //   if self.contains(PulseEnvelope::SPRITE_PATTERN_ADDR) {
  //     0x1000
  //   } else {
  //     0
  //   }
  // }

  // pub fn update(&mut self, data: u8) {
  //   self.bits = data;
  // }
}
