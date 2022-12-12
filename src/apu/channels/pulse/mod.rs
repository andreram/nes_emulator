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

  pub fn write_to_envelope(&mut self, data: u8) {
    self.envelope.update(data)
  }

  pub fn write_to_sweep(&mut self, data: u8) {
    self.sweep.update(data)
  }

  pub fn write_to_timer(&mut self, data: u8) {
    self.timer.update(data)
  }

  pub fn write_to_length(&mut self, data: u8) {
    self.length.update(data)
  }

  pub fn read_length_counter(&self) -> u8 {
    self.length.read_length()
  }
}
