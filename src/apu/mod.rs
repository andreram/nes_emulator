pub mod channels;
pub mod mixer;
pub mod status;
pub mod divider;

use channels::pulse::PulseRegister;
use mixer::APUMixer;
use status::APUStatus;

use self::mixer::APUMixerOutputs;

const CPU_CLOCK_RATE_HZ: i32 = 1789773;
const FRAME_COUNTER_CLOCK_RATE_HZ: i32 = 240;
const CYCLES_PER_FRAME_COUNTER_CLOCK: i32 = CPU_CLOCK_RATE_HZ / FRAME_COUNTER_CLOCK_RATE_HZ;

pub struct APU {
  pulse: PulseRegister,
  pulse_2: PulseRegister,
  status: APUStatus,
  mixer: APUMixer,
  cpu_cycles: usize,
  frame_counter: FrameCounter,
}

bitflags! {
  pub struct FrameCounterByte: u8 {
    const IRQ_INHIBIT       = 0b01000000;
    const MODE              = 0b10000000;
  }
}

impl FrameCounterByte {
  pub fn new() -> Self {
    FrameCounterByte::from_bits_truncate(0)
  }

  pub fn update(&mut self, data: u8) {
    self.bits = data;
  }
}

pub struct FrameCounter {
  data: FrameCounterByte,
  apu_cycles: u32,
}

impl FrameCounter {
  pub fn new() -> Self {
    FrameCounter {
      data: FrameCounterByte::new(),
      apu_cycles: 0,
    }
  }

  pub fn sequence(&mut self) {
    if self.data.contains(FrameCounterByte::MODE) {
      self.sequence_mode_1()
    }
    self.sequence_mode_0()
  }

  fn sequence_mode_0(&mut self) {
    let step = self.get_sequence_step();

    if step == 1 || step == 3 {
      // clock envelopes
    } else if step == 2 || step == 4 {
      // clock envelopes, length counters and sweeps

      if step == 4 {
        // set frame interrupt flag if IRQ_INHIBIT is clear
        // reset apu_cycles to 0
      }
    }
  }

  
  fn sequence_mode_1(&mut self) {
    let step = self.get_sequence_step();

    if step == 1 || step == 3 {
      // clock envelopes
    } else if step == 2 || step == 5 {
      // clock envelopes, length counters and sweeps

      if step == 5 {
        // reset apu_cycles to 0
      }
    }
  }

  fn get_sequence_step(&mut self) -> u8 {
    match self.apu_cycles {
      c if c > 18640 => 5,
      c if c > 14914 => 4,
      c if c > 11185 => 3,
      c if c > 7456 => 2,
      c if c > 3728 => 1,
      _ => panic!("Trying to get sequence step but not enough APU cycles have elapsed!")
    }
  }
}


impl APU {
  pub fn new() -> Self {
    APU {
      pulse: PulseRegister::new(),
      pulse_2: PulseRegister::new(),
      status: APUStatus::new(),
      mixer: APUMixer::new(),
      cpu_cycles: 0,
      frame_counter: FrameCounter::new(),
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
    let silence_pulse_1 = data & 0b1 != 0;
    let silence_pulse_2 = data & 0b10 != 0;

    self.status.update(data);

    if silence_pulse_1 {
      self.pulse.silence_channel();
    }

    if silence_pulse_2 {
      self.pulse_2.silence_channel();
    }
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

  pub fn get_audio_sample(&mut self) {
    self.mixer.get_output(&APUMixerOutputs {
      pulse_1: self.pulse.get_output(),
      pulse_2: self.pulse_2.get_output(),
      triangle: 0,
      noise: 0,
      dmc: 0,
    });

    // TODO: Add first-order filters to sample
  }

  fn clock_timers(&mut self) {
    // TODO: Add clocking for other channels
    if self.cpu_cycles % 2 == 0 {
      self.pulse.clock_sequencer();
      self.pulse_2.clock_sequencer();
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    for _ in 0..cycles {
      self.cpu_cycles += 1;
      self.clock_timers();
    }



  }
}
