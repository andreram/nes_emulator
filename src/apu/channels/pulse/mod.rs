pub mod envelope;
pub mod length;
pub mod sweep;
pub mod timer;

use envelope::PulseEnvelope;
use length::PulseLength;
use sweep::PulseSweep;
use timer::PulseTimer;

pub struct PulseRegister {
  envelope: PulseEnvelope,
  length: PulseLength,
  sweep: PulseSweep,
  timer: PulseTimer,
}

impl PulseRegister {
  pub fn new() -> Self {
    PulseRegister { 
      envelope: PulseEnvelope::new(),
      length: PulseLength::new(),
      sweep: PulseSweep::new(),
      timer: PulseTimer::new(),
    }
  }
}
