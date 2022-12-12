pub mod channels;
pub mod status;

use channels::pulse::PulseRegister;
use status::APUStatus;

pub struct APU {
  pulse: PulseRegister,
  pulse_2: PulseRegister,
  status: APUStatus,
}

impl APU {
  pub fn new() -> Self {
    APU {
      pulse: PulseRegister::new(),
      pulse_2: PulseRegister::new(),
      status: APUStatus::new(),
    }
  }

  pub fn write_to_pulse_envelope(&mut self, data: u8) {
    self.pulse.write_to_envelope(data)
  }

  pub fn write_to_pulse_sweep(&mut self, data: u8) {
    self.pulse.write_to_sweep(data)
  }

  pub fn write_to_pulse_timer(&mut self, data: u8) {
    self.pulse.write_to_timer(data)
  }

  pub fn write_to_pulse_length(&mut self, data: u8) {
    // TODO: restart envelope and reset phase of pulse generator
    self.pulse.write_to_length(data)
  }

  pub fn write_to_pulse_2_envelope(&mut self, data: u8) {
    self.pulse_2.write_to_envelope(data)
  }

  pub fn write_to_pulse_2_sweep(&mut self, data: u8) {
    self.pulse_2.write_to_sweep(data)
  }

  pub fn write_to_pulse_2_timer(&mut self, data: u8) {
    self.pulse_2.write_to_timer(data)
  }

  pub fn write_to_pulse_2_length(&mut self, data: u8) {
    self.pulse_2.write_to_length(data)
  }

  pub fn write_to_status(&mut self, data: u8) {
    self.status.update(data)
  }

  pub fn read_status(&self) -> u8 {
    let mut status: u8 = 0;

    if self.pulse.read_length_counter() > 0 {
      status |= 0b1;
    }

    if self.pulse_2.read_length_counter() > 0 {
      status |= 0b10;
    }

    status
  }
}
