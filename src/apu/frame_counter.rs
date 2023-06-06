bitflags! {
    pub struct FrameCounterByte: u8 {
      const IRQ_INHIBIT       = 0b01000000;
      const MODE              = 0b10000000;
    }
  }
  
  impl FrameCounterByte {
    pub fn new() -> Self {
      FrameCounterByte::from_bits_truncate(0)
    }
  
    pub fn update(&mut self, data: u8) {
      self.bits = data;
    }
  }
  
  pub struct FrameCounter {
    data: FrameCounterByte,
    apu_cycles: u32,
  }
  
  impl FrameCounter {
    pub fn new() -> Self {
      FrameCounter {
        data: FrameCounterByte::new(),
        apu_cycles: 0,
      }
    }
  
    pub fn sequence(&mut self) {
      if self.data.contains(FrameCounterByte::MODE) {
        self.sequence_mode_1()
      }
      self.sequence_mode_0()
    }
  
    fn sequence_mode_0(&mut self) {
      let step = self.get_sequence_step();
  
      if step == 1 || step == 3 {
        // clock envelopes
      } else if step == 2 || step == 4 {
        // clock envelopes, length counters and sweeps
  
        if step == 4 {
          // set frame interrupt flag if IRQ_INHIBIT is clear
          // reset apu_cycles to 0
        }
      }
    }
  
    
    fn sequence_mode_1(&mut self) {
      let step = self.get_sequence_step();
  
      if step == 1 || step == 3 {
        // clock envelopes
      } else if step == 2 || step == 5 {
        // clock envelopes, length counters and sweeps
  
        if step == 5 {
          // reset apu_cycles to 0
        }
      }
    }
  
    fn get_sequence_step(&mut self) -> u8 {
      match self.apu_cycles {
        c if c > 18640 => 5,
        c if c > 14914 => 4,
        c if c > 11185 => 3,
        c if c > 7456 => 2,
        c if c > 3728 => 1,
        _ => panic!("Trying to get sequence step but not enough APU cycles have elapsed!")
      }
    }
  }
  