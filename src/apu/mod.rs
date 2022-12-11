pub mod channels;
pub mod status;

use channels::pulse::PulseRegister;
use status::APUStatus;

pub struct APU {
  pulse: PulseRegister,
  status: APUStatus,
}

impl APU {
  pub fn new() -> Self {
    APU {
      pulse: PulseRegister::new(),
      status: APUStatus::new(),
    }
  }
}
