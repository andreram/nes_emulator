pub struct AddrRegister {
  value: (u8, u8),
  hi_ptr: bool,
}

impl AddrRegister {
  pub fn new() -> Self {
    AddrRegister {
      value: (0, 0), // hi first, lo second
      hi_ptr: true,
    }
  }

  fn set(&mut self, data: u16) {
    self.value.0 = (data >> 8) as u8;
    self.value.1 = (data & 0xff) as u8;
  }

  pub fn update(&mut self, data: u8) {
    if self.hi_ptr {
      self.value.0 = data;
    } else {
      self.value.1 = data;
    }

    // mirror down addr above 0x3fff
    if self.get() > 0x3fff {
      self.set(self.get() & 0x3fff);
    }

    self.hi_ptr = !self.hi_ptr;
  }

  pub fn increment(&mut self, inc: u8) {
    let lo = self.value.1;
    self.value.1 = self.value.1.wrapping_add(inc);

    if lo > self.value.1 {
      self.value.0 = self.value.0.wrapping_add(1);
    }

    // mirror down addr above 0x3fff
    if self.get() > 0x3fff {
      self.set(self.get() & 0x3fff);
    }
  }

  pub fn reset_latch(&mut self) {
    self.hi_ptr = true;
  }

  pub fn get(&self) -> u16 {
    (self.value.0 as u16) << 8 | self.value.1 as u16
  }
}