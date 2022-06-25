pub struct ScrollRegister {
  horizontal_offset: u8,
  vertical_offset: u8,
  horizontal_ptr: bool,
}

impl ScrollRegister {
  pub fn new() -> Self {
    ScrollRegister {
      horizontal_offset: 0,
      vertical_offset: 0,
      horizontal_ptr: true,
    }
  }

  pub fn update(&mut self, data: u8) {
    if self.horizontal_ptr {
      self.horizontal_offset = data;
    } else {
      self.vertical_offset = data;
    }

    self.horizontal_ptr = !self.horizontal_ptr;
  }

  pub fn reset_latch(&mut self) {
    self.horizontal_ptr = true;
  }
}