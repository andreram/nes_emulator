use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct Op {
  pub code: u8,
  pub ins: &'static str,
  pub len: u8,
  pub cycles: u8,
  pub mode: AddressingMode,
}

static OPS: [Op; 227] = [
  Op {code: 0x00, ins: "BRK", len: 1, cycles: 7, mode: AddressingMode::NoneAddressing},

  Op {code: 0x69, ins: "ADC", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0x65, ins: "ADC", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x75, ins: "ADC", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x6d, ins: "ADC", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x7d, ins: "ADC", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x79, ins: "ADC", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0x61, ins: "ADC", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x71, ins: "ADC", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},

  Op {code: 0x29, ins: "AND", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0x25, ins: "AND", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x35, ins: "AND", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x2d, ins: "AND", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x3d, ins: "AND", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x39, ins: "AND", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0x21, ins: "AND", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x31, ins: "AND", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},

  Op {code: 0x0a, ins: "ASL", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x06, ins: "ASL", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x16, ins: "ASL", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x0e, ins: "ASL", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x1e, ins: "ASL", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x90, ins: "BCC", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xb0, ins: "BCS", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xf0, ins: "BEQ", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x30, ins: "BMI", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xd0, ins: "BNE", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x10, ins: "BPL", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x50, ins: "BVC", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x70, ins: "BVS", len: 2, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0x24, ins: "BIT", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x2c, ins: "BIT", len: 3, cycles: 4, mode: AddressingMode::Absolute},

  Op {code: 0x18, ins: "CLC", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xd8, ins: "CLD", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x58, ins: "CLI", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xb8, ins: "CLV", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0xc9, ins: "CMP", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xc5, ins: "CMP", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xd5, ins: "CMP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xcd, ins: "CMP", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xdd, ins: "CMP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0xd9, ins: "CMP", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0xc1, ins: "CMP", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0xd1, ins: "CMP", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},

  Op {code: 0xe0, ins: "CPX", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xe4, ins: "CPX", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xec, ins: "CPX", len: 3, cycles: 4, mode: AddressingMode::Absolute},

  Op {code: 0xc0, ins: "CPY", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xc4, ins: "CPY", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xcc, ins: "CPY", len: 3, cycles: 4, mode: AddressingMode::Absolute},

  Op {code: 0xc6, ins: "DEC", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0xd6, ins: "DEC", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xce, ins: "DEC", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0xde, ins: "DEC", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0xca, ins: "DEX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x88, ins: "DEY", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0x49, ins: "EOR", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0x45, ins: "EOR", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x55, ins: "EOR", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x4d, ins: "EOR", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x5d, ins: "EOR", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x59, ins: "EOR", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0x41, ins: "EOR", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x51, ins: "EOR", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},

  Op {code: 0xe6, ins: "INC", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0xf6, ins: "INC", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xee, ins: "INC", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0xfe, ins: "INC", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0xe8, ins: "INX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xc8, ins: "INY", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0x4c, ins: "JMP", len: 3, cycles: 3, mode: AddressingMode::NoneAddressing}, // AddressingMode that acts as immediate
  Op {code: 0x6c, ins: "JMP", len: 3, cycles: 5, mode: AddressingMode::NoneAddressing}, // AddressingMode::Indirect with 6502 bug
  
  Op {code: 0x20, ins: "JSR", len: 3, cycles: 6, mode: AddressingMode::NoneAddressing},

  Op {code: 0xa9, ins: "LDA", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xa5, ins: "LDA", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xb5, ins: "LDA", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xad, ins: "LDA", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xbd, ins: "LDA", len: 3, cycles: 4 /*+1 if page crossed*/, mode: AddressingMode::Absolute_X},
  Op {code: 0xb9, ins: "LDA", len: 3, cycles: 4 /*+1 if page crossed*/, mode: AddressingMode::Absolute_Y},
  Op {code: 0xa1, ins: "LDA", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0xb1, ins: "LDA", len: 2, cycles: 5 /*+1 if page crossed*/, mode: AddressingMode::Indirect_Y},

  Op {code: 0xa2, ins: "LDX", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xa6, ins: "LDX", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xb6, ins: "LDX", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_Y},
  Op {code: 0xae, ins: "LDX", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xbe, ins: "LDX", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},

  Op {code: 0xa0, ins: "LDY", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xa4, ins: "LDY", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xb4, ins: "LDY", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xac, ins: "LDY", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xbc, ins: "LDY", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},

  Op {code: 0x4a, ins: "LSR", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x46, ins: "LSR", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x56, ins: "LSR", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x4e, ins: "LSR", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x5e, ins: "LSR", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0xea, ins: "NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x1a, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x3a, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x5a, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x7a, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xda, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xfa, ins: "*NOP", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x80, ins: "*NOP", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0x04, ins: "*NOP", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x44, ins: "*NOP", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x64, ins: "*NOP", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x0c, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x14, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x34, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x54, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x74, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xd4, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xf4, ins: "*NOP", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x1c, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x3c, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x5c, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x7c, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0xdc, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0xfc, ins: "*NOP", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},

  Op {code: 0x09, ins: "ORA", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0x05, ins: "ORA", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x15, ins: "ORA", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x0d, ins: "ORA", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x1d, ins: "ORA", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0x19, ins: "ORA", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0x01, ins: "ORA", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x11, ins: "ORA", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},

  Op {code: 0x48, ins: "PHA", len: 1, cycles: 3, mode: AddressingMode::NoneAddressing},
  Op {code: 0x08, ins: "PHP", len: 1, cycles: 3, mode: AddressingMode::NoneAddressing},
  Op {code: 0x68, ins: "PLA", len: 1, cycles: 4, mode: AddressingMode::NoneAddressing},
  Op {code: 0x28, ins: "PLP", len: 1, cycles: 4, mode: AddressingMode::NoneAddressing},

  Op {code: 0x2a, ins: "ROL", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x26, ins: "ROL", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x36, ins: "ROL", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x2e, ins: "ROL", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x3e, ins: "ROL", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x6a, ins: "ROR", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x66, ins: "ROR", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x76, ins: "ROR", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x6e, ins: "ROR", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x7e, ins: "ROR", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x40, ins: "RTI", len: 1, cycles: 6, mode: AddressingMode::NoneAddressing},
  Op {code: 0x60, ins: "RTS", len: 1, cycles: 6, mode: AddressingMode::NoneAddressing},

  Op {code: 0xe9, ins: "SBC", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xe5, ins: "SBC", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xf5, ins: "SBC", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xed, ins: "SBC", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xfd, ins: "SBC", len: 3, cycles: 4, mode: AddressingMode::Absolute_X},
  Op {code: 0xf9, ins: "SBC", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},
  Op {code: 0xe1, ins: "SBC", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0xf1, ins: "SBC", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},
  Op {code: 0xeb, ins: "*SBC", len: 2, cycles: 2, mode: AddressingMode::Immediate},

  Op {code: 0x38, ins: "SEC", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xf8, ins: "SED", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x78, ins: "SEI", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0x85, ins: "STA", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x95, ins: "STA", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x8d, ins: "STA", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x9d, ins: "STA", len: 3, cycles: 5, mode: AddressingMode::Absolute_X},
  Op {code: 0x99, ins: "STA", len: 3, cycles: 5, mode: AddressingMode::Absolute_Y},
  Op {code: 0x81, ins: "STA", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x91, ins: "STA", len: 2, cycles: 6, mode: AddressingMode::Indirect_Y},

  Op {code: 0x86, ins: "STX", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x96, ins: "STX", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_Y},
  Op {code: 0x8e, ins: "STX", len: 3, cycles: 4, mode: AddressingMode::Absolute},

  Op {code: 0x84, ins: "STY", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x94, ins: "STY", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x8c, ins: "STY", len: 3, cycles: 4, mode: AddressingMode::Absolute},

  Op {code: 0xaa, ins: "TAX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xa8, ins: "TAY", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xba, ins: "TSX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x8a, ins: "TXA", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x9a, ins: "TXS", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0x98, ins: "TYA", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0xa3, ins: "*LAX", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0xa7, ins: "*LAX", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xaf, ins: "*LAX", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xb3, ins: "*LAX", len: 2, cycles: 5, mode: AddressingMode::Indirect_Y},
  Op {code: 0xb7, ins: "*LAX", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_Y},
  Op {code: 0xbf, ins: "*LAX", len: 3, cycles: 4, mode: AddressingMode::Absolute_Y},

  Op {code: 0x83, ins: "*SAX", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x87, ins: "*SAX", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x8f, ins: "*SAX", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x97, ins: "*SAX", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_Y},

  Op {code: 0xc3, ins: "*DCP", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0xc7, ins: "*DCP", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0xcf, ins: "*DCP", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0xd3, ins: "*DCP", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0xd7, ins: "*DCP", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xdb, ins: "*DCP", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0xdf, ins: "*DCP", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0xe3, ins: "*ISB", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0xe7, ins: "*ISB", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0xef, ins: "*ISB", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0xf3, ins: "*ISB", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0xf7, ins: "*ISB", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xfb, ins: "*ISB", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0xff, ins: "*ISB", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x03, ins: "*SLO", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0x07, ins: "*SLO", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x0f, ins: "*SLO", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x13, ins: "*SLO", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0x17, ins: "*SLO", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x1b, ins: "*SLO", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0x1f, ins: "*SLO", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x23, ins: "*RLA", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0x27, ins: "*RLA", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x2f, ins: "*RLA", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x33, ins: "*RLA", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0x37, ins: "*RLA", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x3b, ins: "*RLA", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0x3f, ins: "*RLA", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x43, ins: "*SRE", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0x47, ins: "*SRE", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x4f, ins: "*SRE", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x53, ins: "*SRE", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0x57, ins: "*SRE", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x5b, ins: "*SRE", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0x5f, ins: "*SRE", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},

  Op {code: 0x63, ins: "*RRA", len: 2, cycles: 8, mode: AddressingMode::Indirect_X},
  Op {code: 0x67, ins: "*RRA", len: 2, cycles: 5, mode: AddressingMode::ZeroPage},
  Op {code: 0x6f, ins: "*RRA", len: 3, cycles: 6, mode: AddressingMode::Absolute},
  Op {code: 0x73, ins: "*RRA", len: 2, cycles: 8, mode: AddressingMode::Indirect_Y},
  Op {code: 0x77, ins: "*RRA", len: 2, cycles: 6, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x7b, ins: "*RRA", len: 3, cycles: 7, mode: AddressingMode::Absolute_Y},
  Op {code: 0x7f, ins: "*RRA", len: 3, cycles: 7, mode: AddressingMode::Absolute_X},
];

lazy_static! {
  pub static ref OPS_MAP: HashMap<u8, &'static Op> = {
    let mut map = HashMap::new();
    for op in &OPS {
      map.insert(op.code, op);
    }
    map
  };
}
