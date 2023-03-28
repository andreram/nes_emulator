use crate::apu::divider::Divider;

bitflags! {
  struct PulseSweepRegister: u8 {
    const SHIFT   = 0b00000111;
    const NEGATE  = 0b00001000;
    const PERIOD  = 0b01110000;
    const ENABLED = 0b10000000;
  }
}

impl PulseSweepRegister {
  pub fn update(&mut self, data: u8) {
    self.bits = data
  }
}


pub struct PulseSweep {
  register: PulseSweepRegister,
  divider: Divider,
  reload_flag: bool,
}

impl PulseSweep {
  pub fn new() -> Self {
    PulseSweep {
      register: PulseSweepRegister::from_bits_truncate(0),
      divider: Divider::new(),
      reload_flag: false,
    }
  }

  pub fn update(&mut self, data: u8) {
    self.register.update(data)
  }

  pub fn calculate_target_period(&mut self, timer_period: u16, is_pulse_1: bool) {
    let shift_count = self.register.bits & PulseSweepRegister::SHIFT.bits;
    let change_amount = timer_period >> shift_count;

    if self.register.bits & PulseSweepRegister::NEGATE.bits != 0 {
      timer_period.wrapping_sub(change_amount).wrapping_sub(if is_pulse_1 { 1 } else { 0 })
    } else {
      timer_period.wrapping_add(change_amount)
    };
  }
}
