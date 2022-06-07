use crate::cpu::Mem;
use crate::rom::Rom;

pub struct Bus {
  cpu_vram: [u8; 2048],
  rom: Rom,
  // TODO: Remove this
  program_counter: [u8; 2],
}

impl Bus {
  pub fn new(rom: Rom) -> Self {
    Bus {
      cpu_vram: [0; 2048],
      rom: rom,
      program_counter: [0x0, 0x86],
    }
  }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PRG_ROM_MAP: u16 = 0x8000;
const PRG_ROM_MAP_END: u16 = 0xFFFF;
const RAM_MIRROR_MASK: u16 = 0b0000_0111_1111_1111; // 0x0 - 0x7FF
const PPU_MIRROR_MASK: u16 = 0b0010_0000_0000_0111; // 0x2000 - 0x2007
const PROGRAM_COUNTER_LO: u16 = 0xFFFC;
const PROGRAM_COUNTER_HI: u16 = 0xFFFD;

impl Mem for Bus {
  fn mem_read(&self, addr: u16) -> u8 {
    match addr {
      RAM ..= RAM_MIRRORS_END => {
        let mirror_down_addr = addr & RAM_MIRROR_MASK;
        self.cpu_vram[mirror_down_addr as usize]
      },
      PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => {
        let _mirror_down_addr = addr & PPU_MIRROR_MASK;
        todo!("PPU is not supported yet")
      },
      PROGRAM_COUNTER_LO => self.program_counter[0],
      PROGRAM_COUNTER_HI => self.program_counter[1],
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => self.read_prg_rom(addr),
      _ => {
        println!("Ignoring mem access at {}", addr);
        0
      }
    }
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    match addr {
      RAM ..= RAM_MIRRORS_END => {
        let mirror_down_addr = addr & RAM_MIRROR_MASK;
        self.cpu_vram[mirror_down_addr as usize] = data;
      },
      PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => {
        let _mirror_down_addr = addr & PPU_MIRROR_MASK;
        todo!("PPU is not supported yet");
      },
      PROGRAM_COUNTER_LO => self.program_counter[0] = data,
      PROGRAM_COUNTER_HI => self.program_counter[1] = data,
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => {
        panic!("Attempt to write cartridge ROM space");
      }
      _ => {
        println!("Ignoring mem write at {}", addr);
      }
    }
  }
}

impl Bus {
  fn read_prg_rom(&self, mut addr: u16) -> u8 {
    addr -= 0x8000;

    // Mirror if needed (rom length is 16KB an )
    if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      addr = addr % 0x4000;
    }

    self.rom.prg_rom[addr as usize]
  }
}
