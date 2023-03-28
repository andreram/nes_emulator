pub struct Divider {
  period_reload_val: u8,
  counter: u8,
}
  
impl Divider {
  pub fn new() -> Self {
    Divider { period_reload_val: 0, counter: 0 }
  }
  
  pub fn clock(&mut self) {
    if (self.counter == 0) {
      self.counter = self.period_reload_val;
    } else {
      self.counter -= 1;
    }
  }
}
