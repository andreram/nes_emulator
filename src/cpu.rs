use crate::ops::OPS_MAP;
use crate::bus::Bus;
use crate::rom::Rom;
use crate::ppu::PPU;
use crate::joypad::Joypad;

pub struct CPU<'a> {
  pub register_a: u8,
  pub register_x: u8,
  pub register_y: u8,
  pub status: u8,
  pub program_counter: u16,
  pub stack_pointer: u8, 
  pub bus: Bus<'a>,
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
const F_BREAK: u8 = 0b0011_0000;
const F_BREAK_BIT_4: u8 = 0b0001_0000;
const F_BREAK_BIT_5: u8 = 0b0010_0000;

const STACK_OFFSET: u16 = 0x100;
const STACK_RESET: u8 = 0xfd;

pub trait Mem {
  fn mem_read(&mut self, addr: u16) -> u8;

  fn mem_write(&mut self, addr: u16, data: u8);

  fn mem_read_u16(&mut self, pos: u16) -> u16 {
    u16::from_le_bytes([self.mem_read(pos), self.mem_read(pos + 1)])
  }

  fn mem_write_u16(&mut self, pos: u16, data: u16) {
    let bytes = data.to_le_bytes();
    self.mem_write(pos, bytes[0]);
    self.mem_write(pos + 1, bytes[1]);
  }
}

impl<'a> Mem for CPU<'a> {
  fn mem_read(&mut self, addr: u16) -> u8 {
    self.bus.mem_read(addr)
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    self.bus.mem_write(addr, data);
  }
}

fn page_crossed(addr1: u16, addr2: u16) -> bool {
  addr1 & 0xFF00 != addr2 & 0xFF00
}

impl<'a> CPU<'a> {

  pub fn new(rom: Rom) -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: 0, // TODO: Change to 0x24 and fix tests
      program_counter: 0,
      stack_pointer: STACK_RESET,
      bus: Bus::new(rom, |_: &PPU, _: &mut Joypad| {}),
    }
  }

  pub fn new_with_gameloop<F>(rom: Rom, gameloop_callback: F) -> Self
  where
    F: FnMut(&PPU, &mut Joypad) + 'a,
  {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: 0,
      program_counter: 0,
      stack_pointer: STACK_RESET,
      bus: Bus::new(rom, gameloop_callback),
    }
  }

  pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> u16 {
    match mode {
      AddressingMode::Immediate => addr,

      AddressingMode::ZeroPage => self.mem_read(addr) as u16,

      AddressingMode::Absolute => self.mem_read_u16(addr),

      AddressingMode::ZeroPage_X => {
        let pos = self.mem_read(addr);
        pos.wrapping_add(self.register_x) as u16
      }

      AddressingMode::ZeroPage_Y => {
        let pos = self.mem_read(addr);
        pos.wrapping_add(self.register_y) as u16
      }

      AddressingMode::Absolute_X => {
        let base = self.mem_read_u16(addr);
        base.wrapping_add(self.register_x as u16)
      }

      AddressingMode::Absolute_Y => {
        let base = self.mem_read_u16(addr);
        base.wrapping_add(self.register_y as u16)
      }

      AddressingMode::Indirect_X => {
        let base = self.mem_read(addr);

        let ptr: u8 = (base as u8).wrapping_add(self.register_x);
        let lo = self.mem_read(ptr as u16);
        let hi = self.mem_read(ptr.wrapping_add(1) as u16);
        u16::from_le_bytes([lo, hi])
      }

      AddressingMode::Indirect_Y => {
        let base = self.mem_read(addr);

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

  // Side effect: Adds a cycle if a page boundary is crossed
  fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
    match mode {
      AddressingMode::Immediate => self.program_counter,
      AddressingMode::Absolute_X | AddressingMode::Absolute_Y => {
        let base = self.mem_read_u16(self.program_counter);
        let addr = self.get_absolute_address(mode, self.program_counter);

        if page_crossed(base, addr) {
          self.bus.tick(1);
        }

        addr
      },
      AddressingMode::Indirect_Y => {
        let base = self.mem_read(self.program_counter);

        let lo = self.mem_read(base as u16);
        let hi = self.mem_read(base.wrapping_add(1) as u16);
        let deref_base = u16::from_le_bytes([lo, hi]);

        let addr = self.get_absolute_address(mode, self.program_counter);

        if page_crossed(deref_base, addr) {
          self.bus.tick(1);
        }

        addr
      }
      _ => self.get_absolute_address(mode, self.program_counter),
    }
  }

  fn set_register_a(&mut self, value: u8) {
    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  fn stack_push(&mut self, data: u8) {
    self.mem_write(STACK_OFFSET + (self.stack_pointer as u16), data);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
  }

  fn stack_pop(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.mem_read(STACK_OFFSET + (self.stack_pointer as u16))
  }

  fn stack_push_u16(&mut self, data: u16) {
    let bytes = data.to_le_bytes();
    self.stack_push(bytes[1]);
    self.stack_push(bytes[0]);
  }

  fn stack_pop_u16(&mut self) -> u16 {
    let lo = self.stack_pop();
    let hi = self.stack_pop();
    u16::from_le_bytes([lo, hi])
  }


  // Load functions

  pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;
    self.stack_pointer = STACK_RESET;

    // self.status = (self.status & !F_INT) & !F_BREAK;
    // TODO: Remove magic number
    self.status = 0x24;

    // Switch lines for tests 
    self.program_counter = self.mem_read_u16(0xFFFC);
    // self.program_counter = 0x0600;
  }

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  pub fn load(&mut self, program: Vec<u8>) {
    for i in 0..program.len() {
      self.mem_write(0x600 + i as u16, program[i]);
    }

    // self.mem_write_u16(0xFFFC, 0x600);
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

  fn update_flag(&mut self, flag: u8, set_flag: bool) {
    if set_flag {
      self.status = self.status | flag;
    } else {
      self.status = self.status & !flag;
    }
  }

  fn add_to_register_a(&mut self, value: u8) {
    let carry_in = (self.status & F_CARRY != 0) as u8; 
    let (sum, overflow) = self.register_a.overflowing_add(value);
    let (result, overflow_with_carry) = sum.overflowing_add(carry_in);

    self.update_flag(F_CARRY, overflow || overflow_with_carry);

    // Checks if the sign bits of the operands are different from the result
    // i.e. two positive numbers added to a negative number or vice versa
    self.update_flag(F_OVRFLW, (value ^ result) & (self.register_a ^ result) & 0x80 != 0);
    self.set_register_a(result);
  }

  // Op functions

  // Arithmetic and logic instructions
  fn adc(&mut self, mode: &AddressingMode) {
    /* Ignoring decimal mode */
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.add_to_register_a(value);
  }

  fn sbc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    // SBC = A - M - (1 - C)
    //     = A - M - 1 + C
    //     = A + (!M + 1) - 1 + C (two's complement)
    //     = A + !M + C
    self.add_to_register_a(!value);
  }

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
    self.update_flag(F_CARRY, value >> 7 == 1);
  }

  fn asl_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value << 1;
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_flag(F_CARRY, value >> 7 == 1);
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
    self.update_flag(F_CARRY, value & 1 == 1);
  }

  fn lsr_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let result = value >> 1;
    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_flag(F_CARRY, value & 1 == 1);
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
    self.update_flag(F_CARRY, reg >= value);
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

  fn bit(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.update_flag(F_NEG, value & 0b1000_0000 != 0);
    self.update_flag(F_OVRFLW, value & 0b0100_0000 != 0);
    self.update_zero_flag(self.register_a & value);
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

  fn tsx(&mut self) {
    self.register_x = self.stack_pointer;
    self.update_zero_and_negative_flags(self.register_x);
  }

  fn txa(&mut self) {
    self.set_register_a(self.register_x);
  }

  fn txs(&mut self) {
    self.stack_pointer = self.register_x;
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
    self.update_flag(F_CARRY, carry == 1);
  }

  fn rol_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let carry = value >> 7;
    let old_carry = self.status & F_CARRY != 0;
    let result = (value << 1) | old_carry as u8;

    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_flag(F_CARRY, carry == 1);
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
    self.update_flag(F_CARRY, carry == 1);
  }

  fn ror_helper(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let carry = value & 1;
    let old_carry = self.status & F_CARRY != 0;
    let result = (value >> 1) | ((old_carry as u8) << 7);

    self.mem_write(addr, result);
    self.update_zero_and_negative_flags(result);
    self.update_flag(F_CARRY, carry == 1);
  }

  fn ror(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => self.ror_accumulator(),
      _ => self.ror_helper(mode)
    }
  }

  fn lax(&mut self, mode: &AddressingMode) {
    self.lda(mode);
    self.tax();
  }

  fn sax(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a & self.register_x);
  }

  fn dcp(&mut self, mode: &AddressingMode) {
    self.dec(mode);
    self.compare(mode, self.register_a);
  }

  fn isb(&mut self, mode: &AddressingMode) {
    self.inc(mode);
    self.sbc(mode);
  }

  fn slo(&mut self, mode: &AddressingMode) {
    self.asl(mode);
    self.or(mode);
  }

  fn rla(&mut self, mode: &AddressingMode) {
    self.rol(mode);
    self.and(mode);
  }

  fn sre(&mut self, mode: &AddressingMode) {
    self.lsr(mode);
    self.eor(mode);
  }

  fn rra(&mut self, mode: &AddressingMode) {
    self.ror(mode);
    self.adc(mode);
  }

  // Branch control instructions
  fn branch(&mut self, flag: u8, branch_on_set: bool) {
    let flag_set = self.status & flag != 0;

    if flag_set == branch_on_set {
      self.bus.tick(1);

      // Casted as i8 for signed extension when casted to u16
      let offset = self.mem_read(self.program_counter) as i8;

      let jump_addr = self.program_counter
        .wrapping_add(1)
        .wrapping_add(offset as u16);

      if page_crossed(self.program_counter.wrapping_add(1), jump_addr) {
        self.bus.tick(1);
      }

      self.program_counter = jump_addr;
    }
  }

  fn jsr(&mut self) {
    /* Subtract one to undo the PC increment from the opcode */
    self.stack_push_u16(self.program_counter + 2 - 1);
    self.program_counter = self.mem_read_u16(self.program_counter);
  }

  fn rts(&mut self) {
    self.program_counter = self.stack_pop_u16() + 1;
  }

  fn rti(&mut self) {
    self.plp();
    self.program_counter = self.stack_pop_u16();
  }

  // Stack related instructions
  fn pha(&mut self) {
    self.stack_push(self.register_a);
  }

  fn php(&mut self) {
    self.stack_push(self.status | F_BREAK);
  }

  fn pla(&mut self) {
    let data = self.stack_pop();
    self.set_register_a(data);
  }

  fn plp(&mut self) {
    self.status = (self.stack_pop() & !F_BREAK) | (self.status & F_BREAK);
  }

  fn nop(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::NoneAddressing => {},
      AddressingMode::Immediate => {},
      _ => {
        let addr = self.get_operand_address(mode);
        self.mem_read(addr);
      },
    };
  }

  fn interrupt_nmi(&mut self) {
    self.stack_push_u16(self.program_counter);

    // Push status with break flag set to 10
    self.stack_push((self.status | F_BREAK_BIT_5) & !F_BREAK_BIT_4);
    self.sei();

    self.bus.tick(2); // NMI cycles
    self.program_counter = self.mem_read_u16(0xFFFA);
  }

  pub fn run(&mut self) {
    self.run_with_callback(|_| {});
  }

  pub fn run_with_callback<F>(&mut self, mut callback: F)
  where
    F: FnMut(&mut CPU)
  {
    loop {
      if let Some(_) = self.bus.poll_nmi_interrupt() {
        self.interrupt_nmi();
      }

      callback(self);

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

        "TSX" => self.tsx(),

        "TXA" => self.txa(),

        "TXS" => self.txs(),

        "TYA" => self.tya(),

        "INC" => self.inc(&op.mode),

        "INX" => self.inx(),

        "INY" => self.iny(),

        "DEC" => self.dec(&op.mode),

        "DEX" => self.dex(),

        "DEY" => self.dey(),

        "ADC" => self.adc(&op.mode),

        "SBC" => self.sbc(&op.mode),

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

        "BCC" => self.branch(F_CARRY, false),

        "BCS" => self.branch(F_CARRY, true),

        "BNE" => self.branch(F_ZERO, false),

        "BEQ" => self.branch(F_ZERO, true),

        "BPL" => self.branch(F_NEG, false),

        "BMI" => self.branch(F_NEG, true),

        "BVC" => self.branch(F_OVRFLW, false),

        "BVS" => self.branch(F_OVRFLW, true),

        "JMP" => {
          match opcode {
            /* Absolute */
            0x4c => {
              self.program_counter = self.mem_read_u16(self.program_counter);
            },

            /* Indirect */
            0x6c => {
              let addr = self.mem_read_u16(self.program_counter);
              // self.program_counter = self.mem_read_u16(addr);

              // 6502 bug mode with with page boundary:
              //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
              //  the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
              //  i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

              let indirect_addr = if addr & 0x00FF == 0x00FF {
                let lo = self.mem_read(addr);
                let hi = self.mem_read(addr & 0xFF00);
                (hi as u16) << 8 | (lo as u16)
              } else {
                self.mem_read_u16(addr)
              };

              self.program_counter = indirect_addr;
            },

            _ => {},
          }
        },

        "JSR" => self.jsr(),

        "RTS" => self.rts(),

        "BIT" => self.bit(&op.mode),

        "RTI" => self.rti(),

        "PHA" => self.pha(),

        "PHP" => self.php(),

        "PLA" => self.pla(),

        "PLP" => self.plp(),

        "NOP" => {},

        "*NOP" => self.nop(&op.mode),

        "*LAX" => self.lax(&op.mode),

        "*SAX" => self.sax(&op.mode),

        "*SBC" => self.sbc(&op.mode),

        "*DCP" => self.dcp(&op.mode),

        "*ISB" => self.isb(&op.mode),

        "*SLO" => self.slo(&op.mode),

        "*RLA" => self.rla(&op.mode),

        "*SRE" => self.sre(&op.mode),

        "*RRA" => self.rra(&op.mode),

        "BRK" => return,

        _ => panic!("unknown opcode: {}", opcode),
      }

      self.bus.tick(op.cycles);

      if program_counter_state == self.program_counter {
        self.program_counter += (op.len - 1) as u16;
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::rom::test;

  #[test]
  fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xa9_lda_negative_flag() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
    assert!(cpu.status & F_NEG != 0);
  }

  #[test]
  fn test_0xaa_tax() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
    assert_eq!(cpu.register_x, 10);
    assert!(cpu.status & F_ZERO == 0);
    assert!(cpu.status & F_NEG == 0);
  }

  #[test]
  fn test_0xaa_tax_zero_flag() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & F_ZERO != 0);
  }

  #[test]
  fn test_0xaa_tax_negative_flag() {
    let mut cpu = CPU::new(test::test_rom());
    cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
    assert_ne!(cpu.status & F_NEG, 0);
  }

  #[test]
  fn test_5_ops_working_together() {
      let mut cpu = CPU::new(test::test_rom());
      cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

      assert_eq!(cpu.register_x, 0xc1)
  }

   #[test]
   fn test_inx_overflow() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);

       assert_eq!(cpu.register_x, 0);
       assert_ne!(cpu.status & F_ZERO, 0);
   }

   #[test]
   fn test_lda_from_memory() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

       assert_eq!(cpu.register_a, 0x55);
   }

   #[test]
   fn test_sta() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.mem_write(0x10, 0x55);

       cpu.load_and_run(vec![0xa5, 0x10, 0x85, 0x20, 0x00]);

       let value = cpu.mem_read(0x20 as u16);
       assert_eq!(value, 0x55);
   }

   #[test]
   fn test_adc() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x05, 0x69, 0x01, 0x00]);
       assert_eq!(cpu.register_a, 0x6);
   }

   #[test]
   fn test_adc_with_carry() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x05, 0x38, 0x69, 0x01, 0x00]);
       assert_eq!(cpu.register_a, 0x7);
   }

   // The next 8 tests are pulled from the table here:
   // https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
   #[test]
   fn test_adc_1() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0x69, 0x10, 0x00]);
       assert_eq!(cpu.register_a, 0x60);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_2() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0x69, 0x50, 0x00]);
       assert_eq!(cpu.register_a, 0xa0);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_3() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0x69, 0x90, 0x00]);
       assert_eq!(cpu.register_a, 0xe0);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_4() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0x69, 0xd0, 0x00]);
       assert_eq!(cpu.register_a, 0x20);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_5() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x69, 0x10, 0x00]);
       assert_eq!(cpu.register_a, 0xe0);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_6() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x69, 0x50, 0x00]);
       assert_eq!(cpu.register_a, 0x20);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_7() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x69, 0x90, 0x00]);
       assert_eq!(cpu.register_a, 0x60);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_adc_8() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x69, 0xd0, 0x00]);
       assert_eq!(cpu.register_a, 0xa0);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x05, 0xe9, 0x01, 0x00]);
       assert_eq!(cpu.register_a, 0x3);
   }

   #[test]
   fn test_sbc_with_carry() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x05, 0x38, 0xe9, 0x01, 0x00]);
       assert_eq!(cpu.register_a, 0x4);
   }

   #[test]
   fn test_sbc_1() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0xe9, 0xf0, 0x00]);
       assert_eq!(cpu.register_a, 0x5f);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_2() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0xe9, 0xb0, 0x00]);
       assert_eq!(cpu.register_a, 0x9f);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_3() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0xe9, 0x70, 0x00]);
       assert_eq!(cpu.register_a, 0xdf);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_4() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x50, 0xe9, 0x30, 0x00]);
       assert_eq!(cpu.register_a, 0x1f);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_5() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x38, 0xe9, 0xf0, 0x00]);
       assert_eq!(cpu.register_a, 0xe0);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_6() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x38, 0xe9, 0xb0, 0x00]);
       assert_eq!(cpu.register_a, 0x20);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_7() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x38, 0xe9, 0x70, 0x00]);
       assert_eq!(cpu.register_a, 0x60);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_sbc_8() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xd0, 0x38, 0xe9, 0x30, 0x00]);
       assert_eq!(cpu.register_a, 0xa0);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_OVRFLW, 0);
   }

   #[test]
   fn test_asl_acc_1() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x01, 0x0a, 0x00]);
       assert_eq!(cpu.register_a, 0x02);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_NEG, 0);
       assert_eq!(cpu.status & F_ZERO, 0);
   }

   #[test]
   fn test_asl_acc_2() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xf1, 0x0a, 0x00]);
       assert_eq!(cpu.register_a, 0xe2);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_NEG, 0);
   }

   #[test]
   fn test_asl_acc_3() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x7f, 0x0a, 0x00]);
       assert_eq!(cpu.register_a, 0xfe);
       assert_eq!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_NEG, 0);
   }

   #[test]
   fn test_asl_acc_4() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x80, 0x0a, 0x00]);
       assert_eq!(cpu.register_a, 0);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_NEG, 0);
       assert_ne!(cpu.status & F_ZERO, 0);
   }

   #[test]
   fn test_lsr_acc_1() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x01, 0x4a, 0x00]);
       assert_eq!(cpu.register_a, 0);
       assert_eq!(cpu.status & F_NEG, 0);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_ne!(cpu.status & F_ZERO, 0);
   }

   #[test]
   fn test_lsr_acc_2() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x81, 0x4a, 0x00]);
       assert_eq!(cpu.register_a, 0x40);
       assert_eq!(cpu.status & F_NEG, 0);
       assert_ne!(cpu.status & F_CARRY, 0);
       assert_eq!(cpu.status & F_ZERO, 0);
   }

   #[test]
   fn test_lsr_acc_3() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x82, 0x4a, 0x00]);
       assert_eq!(cpu.register_a, 0x41);
       assert_eq!(cpu.status & F_CARRY, 0);
   }

   #[test]
   fn test_pha_pla() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0x11, 0x48, 0xa9, 0x22, 0x68, 0x00]);
       assert_eq!(cpu.register_a, 0x11);
   }

   #[test]
   fn test_pha_plp() {
       let mut cpu = CPU::new(test::test_rom());
       cpu.load_and_run(vec![0xa9, 0xff, 0x48, 0x28, 0x00]);
       assert_eq!(cpu.status & !F_BREAK, !F_BREAK);
   }

   #[test]
   fn test_jsr_rts() {
       let mut cpu = CPU::new(test::test_rom());
       /*
          JSR init
          JSR loop
          JSR end

        init:
          LDX #$00
          RTS

        loop:
          INX
          CPX #$05
          BNE loop
          RTS

        end:
          BRK
        */
       cpu.load_and_run(vec![0x20, 0x09, 0x06, 0x20, 0x0c, 0x06, 0x20, 0x12, 0x06, 0xa2, 0x00, 0x60, 0xe8, 0xe0, 0x05, 0xd0, 0xfb, 0x60, 0x00]);
       assert_eq!(cpu.register_x, 5);
       assert_eq!(cpu.status & F_NEG, 0);
       assert_ne!(cpu.status & F_ZERO, 0);
       assert_ne!(cpu.status & F_CARRY, 0);
   }
}
