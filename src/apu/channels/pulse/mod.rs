pub mod envelope;
pub mod length;
pub mod sequencer;
pub mod sweep;
pub mod timer;

use envelope::PulseEnvelope;
use length::PulseLength;
use sequencer::PulseSequencer;
use sweep::PulseSweep;
use timer::PulseTimer;

pub struct PulseRegister {
  envelope: PulseEnvelope,
  length: PulseLength,
  sweep: PulseSweep,
  timer: PulseTimer,
  sequencer: PulseSequencer,
  target_period: u16,
  sweep_reset: bool,
}

const DUTY_CYCLE_TABLE: [[u8; 8]; 4] = [
  [0, 1, 0, 0, 0, 0, 0, 0],
  [0, 1, 1, 0, 0, 0, 0, 0],
  [0, 1, 1, 1, 1, 0, 0, 0],
  [1, 0, 0, 1, 1, 1, 1, 1],
];

impl PulseRegister {
  pub fn new() -> Self {
    PulseRegister {
      envelope: PulseEnvelope::new(),
      length: PulseLength::new(),
      sweep: PulseSweep::new(),
      timer: PulseTimer::new(),
      sequencer: PulseSequencer::new(),
      target_period: 0,
      sweep_reset: false,
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

  pub fn read_timer_val(&self) -> u16 {
    self.length.read_timer_high() | ((self.timer.read_timer_low() as u16) & 0xFF)
  }

  pub fn silence_channel(&mut self) {
    self.write_to_length(0)
  }

  pub fn clock_sequencer(&mut self) {
    if self.sequencer.get_divider() > 0 {
      self.sequencer.decrement_divider();
    } else {
      self.sequencer.update_divider(self.read_timer_val());
      self.sequencer.clock_sequence_step();
    }
  }

  pub fn get_output(&self) -> u8 {
    if self.read_length_counter() == 0
      || self.read_timer_val() < 8
      || DUTY_CYCLE_TABLE[self.envelope.get_duty_cycle() as usize]
        [self.sequencer.get_sequence_step() as usize]
        == 0
    {
      0
    } else {
      self.envelope.get_envelope_volume()
    }
  }
}
