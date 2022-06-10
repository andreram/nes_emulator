use crate::cpu::CPU;
use crate::cpu::Mem;
use crate::cpu::AddressingMode;
use crate::ops::OPS_MAP;

pub fn trace(cpu: &mut CPU) -> String {
  let opcode = cpu.mem_read(cpu.program_counter);

  let op = OPS_MAP[&opcode];
  let mut log = String::with_capacity(100);

  log.push_str(&format!("{:04X}  ", &cpu.program_counter));

  let mut args: Vec<String> = vec![];
  for i in 0..op.len as u16 {
    let arg = cpu.mem_read(cpu.program_counter + i);
    let string = format!("{:02X}", arg);
    args.push(string);
  }

  log.push_str(&format!("{:10}", args.join(" ")));
  log.push_str(op.ins);

  let args_assembly = match op.mode {
    AddressingMode::Immediate => {
      format!(" #${}", args[1])
    },
    AddressingMode::ZeroPage => {
      let addr = cpu.mem_read(cpu.program_counter + 1);
      let data = cpu.mem_read(addr as u16);
      format!(" ${} = {:02X}", args[1], data)
    },
    AddressingMode::Indirect_Y => {
      let addr = cpu.mem_read(cpu.program_counter + 1);
      let data = cpu.mem_read_u16(addr as u16);
      let addr = data.wrapping_add(cpu.register_y as u16);
      let data = cpu.mem_read(addr);
      format!(" (${}),Y = {:04X} @ {:04X} = {:02X}", args[1], addr, addr, data)
    },
    AddressingMode::Absolute => {
      let addr = cpu.mem_read_u16(cpu.program_counter + 1);
      let data = cpu.mem_read(addr);
      format!(" ${:04X} = {:02X}", addr, data)
    },
    AddressingMode::Absolute_X => {
      let addr = cpu.mem_read_u16(cpu.program_counter + 1);
      let added_addr = addr.wrapping_add(cpu.register_x as u16);
      let data = cpu.mem_read_u16(added_addr);
      format!(" ${:04X},X @ {:04X} = {:02X}", addr, added_addr, data)
    },
    AddressingMode::Absolute_Y => {
      let addr = cpu.mem_read_u16(cpu.program_counter + 1);
      let added_addr = addr.wrapping_add(cpu.register_y as u16);
      let data = cpu.mem_read_u16(added_addr);
      format!(" ${:04X},Y @ {:04X} = {:02X}", addr, added_addr, data)
    },
    AddressingMode::NoneAddressing => {
      match op.len {
        1 => String::from(""),
        2 => {
          let offset = cpu.mem_read(cpu.program_counter + 1);
          let addr = cpu.program_counter
          .wrapping_add(2)
          .wrapping_add(offset as u16);
          format!(" ${:04X}", addr)
        }
        3 => format!(" ${}{}", args[2], args[1]),
        _ => String::from(""),
      }
    },
    _ => String::from(" todo"),
  };

  log.push_str(&format!("{:29}", args_assembly));

  log.push_str(&format!(
    "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
    cpu.register_a,
    cpu.register_x,
    cpu.register_y,
    cpu.status,
    cpu.stack_pointer,
  ));

  log
}


#[cfg(test)]
mod test {
  use super::*;
  use crate::cpu::CPU;
  use crate::cpu::Mem;
  use crate::rom::test::test_rom;

  #[test]
  fn test_format_trace() {
    let mut cpu = CPU::new(test_rom());
    cpu.mem_write(100, 0xa2);
    cpu.mem_write(101, 0x01);
    cpu.mem_write(102, 0xca);
    cpu.mem_write(103, 0x88);
    cpu.mem_write(104, 0x00);

    cpu.program_counter = 100;
    cpu.register_a = 1;
    cpu.register_x = 2;
    cpu.register_y = 3;
    
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });

    println!("{}", result[0]);

    assert_eq!(
      "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
      result[0]
    );
    assert_eq!(
      "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
      result[1]
    );
    assert_eq!(
      "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
      result[2]
    );
  }

  #[test]
  fn test_format_mem_access() {
    let mut cpu = CPU::new(test_rom());
    // ORA ($33), Y
    cpu.mem_write(100, 0x11);
    cpu.mem_write(101, 0x33);

    // data
    cpu.mem_write(0x33, 0x00);
    cpu.mem_write(0x34, 0x04);

    // target cell
    cpu.mem_write(0x400, 0xAA);

    cpu.program_counter = 100;
    cpu.register_y = 0;

    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });

    assert_eq!(
      "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
      result[0]
    );
  }
}