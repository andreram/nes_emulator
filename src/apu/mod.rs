pub mod channels;
pub mod status;

use channels::pulse::pulse::PulseRegister;
use status::APUStatus;

pub struct APU {
  pulse: PulseRegister,
  status: APUStatus,
}