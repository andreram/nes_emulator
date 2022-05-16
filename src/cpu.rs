pub struct CPU {
  pub register_a: u8,
  pub register_x: u8,
  pub status: u8,
  pub program_counter: u16,
}

const F_ZERO: u8 = 0b0000_0010;
const F_NEG: u8 = 0b1000_0000;

impl CPU {

  pub fn new() -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      status: 0,
      program_counter: 0,
    }
  }

  // Helpers

  fn update_zero_flag(&mut self, result: u8) {
    if result == 0 {
      self.status = self.status | F_ZERO;
    } else {
      self.status = self.status & !F_ZERO;
    }
  }

  fn update_negative_flag(&mut self, result: u8) {
    if result & F_NEG != 0 {
      self.status = self.status | F_NEG;
    } else {
      self.status = self.status & !F_NEG;
    }
  }

  // Op functions

  fn lda(&mut self, value: u8) {
    self.register_a = value;
    self.update_zero_flag(self.register_a);
    self.update_negative_flag(self.register_a);
  }

  fn tax(&mut self) {
    self.register_x = self.register_a;
    self.update_zero_flag(self.register_x);
    self.update_negative_flag(self.register_x);
  }

  pub fn interpret(&mut self, program: Vec<u8>) {
    self.program_counter = 0;

    loop {
      // Fetch next instruction
      let opcode = program[self.program_counter as usize];
      self.program_counter += 1;

      match opcode {
        
        // LDA immediate
        0xA9 => {
          // Load the instruction's parameter into the accumulator
          let param = program[self.program_counter as usize];
          self.program_counter += 1;

          self.lda(param);
        }

        0xAA => self.tax(),

        0x00 => return,

        _ => todo!(),
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xa9_lda_negative_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xa9, 0xff, 0x00]);
    assert!(cpu.status & F_NEG != 0);
  }

  #[test]
  fn test_0xaa_tax() {
    let mut cpu = CPU::new();
    cpu.register_a = 10;
    cpu.interpret(vec![0xaa, 0x00]);
    assert_eq!(cpu.register_x, 10);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xaa_tax_zero_flag() {
    let mut cpu = CPU::new();
    cpu.register_a = 0;
    cpu.interpret(vec![0xaa, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xaa_tax_negative_flag() {
    let mut cpu = CPU::new();
    cpu.register_a = 0xff;
    cpu.interpret(vec![0xaa, 0x00]);
    assert!(cpu.status & F_NEG != 0);
  }
}
