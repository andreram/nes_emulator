use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct Op {
  pub code: u8,
  pub ins: &'static str,
  pub len: u8,
  pub cycles: u8,
  pub mode: AddressingMode,
}

static OPS: [Op; 18] = [
  Op {code: 0x00, ins: "BRK", len: 1, cycles: 7, mode: AddressingMode::NoneAddressing},
  Op {code: 0xaa, ins: "TAX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},
  Op {code: 0xe8, ins: "INX", len: 1, cycles: 2, mode: AddressingMode::NoneAddressing},

  Op {code: 0xa9, ins: "LDA", len: 2, cycles: 2, mode: AddressingMode::Immediate},
  Op {code: 0xa5, ins: "LDA", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0xb5, ins: "LDA", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0xad, ins: "LDA", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0xbd, ins: "LDA", len: 3, cycles: 4 /*+1 if page crossed*/, mode: AddressingMode::Absolute_X},
  Op {code: 0xb9, ins: "LDA", len: 3, cycles: 4 /*+1 if page crossed*/, mode: AddressingMode::Absolute_Y},
  Op {code: 0xa1, ins: "LDA", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0xb1, ins: "LDA", len: 2, cycles: 5 /*+1 if page crossed*/, mode: AddressingMode::Indirect_Y},

  Op {code: 0x85, ins: "STA", len: 2, cycles: 3, mode: AddressingMode::ZeroPage},
  Op {code: 0x95, ins: "STA", len: 2, cycles: 4, mode: AddressingMode::ZeroPage_X},
  Op {code: 0x8d, ins: "STA", len: 3, cycles: 4, mode: AddressingMode::Absolute},
  Op {code: 0x9d, ins: "STA", len: 3, cycles: 5, mode: AddressingMode::Absolute_X},
  Op {code: 0x99, ins: "STA", len: 3, cycles: 5, mode: AddressingMode::Absolute_Y},
  Op {code: 0x81, ins: "STA", len: 2, cycles: 6, mode: AddressingMode::Indirect_X},
  Op {code: 0x91, ins: "STA", len: 2, cycles: 6, mode: AddressingMode::Indirect_Y},
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
