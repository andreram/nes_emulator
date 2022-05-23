use crate::ops::OPS_MAP;

pub struct CPU {
  pub register_a: u8,
  pub register_x: u8,
  pub register_y: u8,
  pub status: u8,
  pub program_counter: u16,
  memory: [u8; 0xFFFF]
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
  Immediate,
  ZeroPage,
  ZeroPage_X,
  ZeroPage_Y,
  Absolute,
  Absolute_X,
  Absolute_Y,
  Indirect_X,
  Indirect_Y,
  NoneAddressing,
}

const F_ZERO: u8 = 0b0000_0010;
const F_NEG: u8 = 0b1000_0000;
const F_CARRY: u8 = 0b0000_0001;
const F_DEC: u8 = 0b0000_1000;
const F_INT: u8 = 0b0000_0100;
const F_OVRFLW: u8 = 0b0100_0000;

impl CPU {

  pub fn new() -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: 0,
      program_counter: 0,
      memory: [0; 0xFFFF]
    }
  }

  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    match mode {
      AddressingMode::Immediate => self.program_counter,

      AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

      AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

      AddressingMode::ZeroPage_X => {
        let pos = self.mem_read(self.program_counter);
        pos.wrapping_add(self.register_x) as u16
      }

      AddressingMode::ZeroPage_Y => {
        let pos = self.mem_read(self.program_counter);
        pos.wrapping_add(self.register_y) as u16
      }

      AddressingMode::Absolute_X => {
        let base = self.mem_read_u16(self.program_counter);
        base.wrapping_add(self.register_x as u16)
      }

      AddressingMode::Absolute_Y => {
        let base = self.mem_read_u16(self.program_counter);
        base.wrapping_add(self.register_y as u16)
      }

      AddressingMode::Indirect_X => {
        let base = self.mem_read(self.program_counter);

        let ptr: u8 = (base as u8).wrapping_add(self.register_x);
        let lo = self.mem_read(ptr as u16);
        let hi = self.mem_read(ptr.wrapping_add(1) as u16);
        u16::from_le_bytes([lo, hi])
      }

      AddressingMode::Indirect_Y => {
        let base = self.mem_read(self.program_counter);

        let lo = self.mem_read(base as u16);
        let hi = self.mem_read(base.wrapping_add(1) as u16);
        let deref_base = u16::from_le_bytes([lo, hi]);
        deref_base.wrapping_add(self.register_y as u16)
      }

      AddressingMode::NoneAddressing => {
        panic!("mode {:?} is not supported", mode);
      }
    }
  }

  fn set_register_a(&mut self, value: u8) {
    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // Memory helpers

  fn mem_read(&self, addr: u16) -> u8 {
    self.memory[addr as usize]
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    self.memory[addr as usize] = data;
  }

  fn mem_read_u16(&self, pos: u16) -> u16 {
    u16::from_le_bytes([self.mem_read(pos), self.mem_read(pos + 1)])
  }

  fn mem_write_u16(&mut self, pos: u16, data: u16) {
    let bytes = data.to_le_bytes();
    self.mem_write(pos, bytes[0]);
    self.mem_write(pos + 1, bytes[1]);
  }

  // Load functions

  pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.status = 0;

    self.program_counter = self.mem_read_u16(0xFFFC);
  }

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  pub fn load(&mut self, program: Vec<u8>) {
    self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x8000);
  }

  // Flag helpers

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

  fn update_zero_and_negative_flags(&mut self, result: u8) {
    self.update_zero_flag(result);
    self.update_negative_flag(result);
  } 

  fn update_carry_flag(&mut self, set_flag: bool) {
    if set_flag {
      self.status = self.status | F_CARRY;
    } else {
      self.status = self.status & !F_CARRY;
    }
  }

  // Op functions

  // Arithmetic and logic instructions
  fn and(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(self.register_a & value);
  }

  fn or(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(self.register_a | value);
  }

  fn eor(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.set_register_a(self.register_a ^ value);
  }

  fn asl_accumulator(&mut self) {
    let value = self.register_a;

    self.set_register_a(value << 1);
    self.update_carry_flag(value >> 7 == 1);
  }

  fn asl_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value << 1;
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_carry_flag(value >> 7 == 1);
  }

  fn asl(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => self.asl_accumulator(),
      _ => self.asl_helper(mode)
    }
  }

  fn lsr_accumulator(&mut self) {
    let value = self.register_a;

    self.set_register_a(value >> 1);
    self.update_carry_flag(value & 1 == 1);
  }

  fn lsr_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value >> 1;
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_carry_flag(value & 1 == 1);
  }

  fn lsr(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => self.lsr_accumulator(),
      _ => self.lsr_helper(mode)
    }
  }

  fn compare(&mut self, mode: &AddressingMode, reg: u8) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.update_zero_and_negative_flags(reg.wrapping_sub(value));
    self.update_carry_flag(reg >= value);
  }

  fn dec(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value.wrapping_sub(1);
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
  }

  fn dex(&mut self) {
    self.register_x = self.register_x.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  fn dey(&mut self) {
    self.register_y = self.register_y.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_y);
  }



  // Status instructions
  fn clc(&mut self) {
    self.status = self.status & !F_CARRY;
  }

  fn cld(&mut self) {
    self.status = self.status & !F_DEC;
  }

  fn cli(&mut self) {
    self.status = self.status & !F_INT;
  }

  fn clv(&mut self) {
    self.status = self.status & !F_OVRFLW;
  }

  fn sec(&mut self) {
    self.status = self.status | F_CARRY;
  }

  fn sed(&mut self) {
    self.status = self.status | F_DEC;
  }

  fn sei(&mut self) {
    self.status = self.status | F_INT;
  }

  fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  fn ldx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_x = value;
    self.update_zero_and_negative_flags(self.register_x);
  }

  fn ldy(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_y = value;
    self.update_zero_and_negative_flags(self.register_y);
  }

  fn sta(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a);
  }

  fn stx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_x);
  }

  fn sty(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_y);
  }

  fn tax(&mut self) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
  }

  fn tay(&mut self) {
    self.register_y = self.register_a;
    self.update_zero_and_negative_flags(self.register_y);
  }

  fn txa(&mut self) {
    self.set_register_a(self.register_x);
  }

  fn tya(&mut self) {
    self.set_register_a(self.register_y);
  }

  fn inc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value.wrapping_add(1);
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
  }

  fn inx(&mut self) {
    self.register_x = self.register_x.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  fn iny(&mut self) {
    self.register_y = self.register_y.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_y);
  }

  fn rol_accumulator(&mut self) {
    let carry = self.register_a >> 7;
    let old_carry = self.status & F_CARRY != 0;
    self.set_register_a((self.register_a << 1) | old_carry as u8);
    self.update_carry_flag(carry == 1);
  }

  fn rol_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let carry = value >> 7;
    let old_carry = self.status & F_CARRY != 0;
    let result = (value << 1) | old_carry as u8;

    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_carry_flag(carry == 1);
  }

  fn rol(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => self.rol_accumulator(),
      _ => self.rol_helper(mode)
    }
  }

  fn ror_accumulator(&mut self) {
    let carry = self.register_a & 1;
    let old_carry = self.status & F_CARRY != 0;
    self.set_register_a((self.register_a >> 1) | ((old_carry as u8) << 7));
    self.update_carry_flag(carry == 1);
  }

  fn ror_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let carry = value & 1;
    let old_carry = self.status & F_CARRY != 0;
    let result = (value >> 1) | ((old_carry as u8) << 7);

    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_carry_flag(carry == 1);
  }

  fn ror(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => self.ror_accumulator(),
      _ => self.ror_helper(mode)
    }
  }

  pub fn run(&mut self) {
    loop {
      // Fetch next instruction
      let opcode = self.mem_read(self.program_counter);
      self.program_counter += 1;
      let program_counter_state = self.program_counter;

      let op = OPS_MAP[&opcode];

      match op.ins {
        
        "LDA" => self.lda(&op.mode),

        "LDX" => self.ldx(&op.mode),

        "LDY" => self.ldy(&op.mode),

        "STA" => self.sta(&op.mode),

        "STX" => self.stx(&op.mode),

        "STY" => self.sty(&op.mode),

        "TAX" => self.tax(),

        "TAY" => self.tay(),

        "TXA" => self.txa(),

        "TYA" => self.tya(),

        "INC" => self.inc(&op.mode),

        "INX" => self.inx(),

        "INY" => self.iny(),

        "DEC" => self.dec(&op.mode),

        "DEX" => self.dex(),

        "DEY" => self.dey(),

        "AND" => self.and(&op.mode),

        "ORA" => self.or(&op.mode),

        "EOR" => self.eor(&op.mode),

        "CMP" => self.compare(&op.mode, self.register_a),

        "CPX" => self.compare(&op.mode, self.register_x),

        "CPY" => self.compare(&op.mode, self.register_y),

        "ROL" => self.rol(&op.mode),

        "ROR" => self.ror(&op.mode),

        "ASL" => self.asl(&op.mode),

        "LSR" => self.lsr(&op.mode),

        "CLC" => self.clc(),

        "CLD" => self.cld(),

        "CLI" => self.cli(),

        "CLV" => self.clv(),

        "SEC" => self.sec(),

        "SED" => self.sed(),

        "SEI" => self.sei(),

        "BRK" => return,

        _ => todo!(),
      }

      if program_counter_state == self.program_counter {
        self.program_counter += (op.len - 1) as u16;
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
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xa9_lda_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
    assert!(cpu.status & F_NEG != 0);
  }

  #[test]
  fn test_0xaa_tax() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
    assert_eq!(cpu.register_x, 10);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xaa_tax_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xaa_tax_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
    assert!(cpu.status & F_NEG != 0);
  }

  #[test]
  fn test_5_ops_working_together() {
      let mut cpu = CPU::new();
      cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

      assert_eq!(cpu.register_x, 0xc1)
  }

   #[test]
   fn test_inx_overflow() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

       assert_eq!(cpu.register_x, 1)
   }

   #[test]
   fn test_lda_from_memory() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

       assert_eq!(cpu.register_a, 0x55);
   }

   #[test]
   fn test_sta() {
       let mut cpu = CPU::new();
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x85, 0x20, 0x00]);

       let value = cpu.mem_read(0x20 as u16);
       assert_eq!(value, 0x55);
   }
}
