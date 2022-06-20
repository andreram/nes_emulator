use crate::cpu::Mem;
use crate::rom::Rom;
use crate::ppu::PPU;

pub struct Bus {
  cpu_vram: [u8; 2048],
  prg_rom: Vec<u8>,
  // TODO: Remove this
  program_counter: [u8; 2],
  ppu: PPU,
}

impl Bus {
  pub fn new(rom: Rom) -> Self {
    let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
    Bus {
      cpu_vram: [0; 2048],
      prg_rom: rom.prg_rom,
      program_counter: [0x0, 0x86],
      ppu: ppu,
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
  fn mem_read(&mut self, addr: u16) -> u8 {
    match addr {
      RAM ..= RAM_MIRRORS_END => {
        let mirror_down_addr = addr & RAM_MIRROR_MASK;
        self.cpu_vram[mirror_down_addr as usize]
      },
      0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
        panic!("Attempted to read from write-only PPU address {:x}", addr);
      },
      0x2007 => self.ppu.read_data(),

      0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & PPU_MIRROR_MASK;
        self.mem_read(mirror_down_addr)
      },
      PROGRAM_COUNTER_LO => self.program_counter[0],
      PROGRAM_COUNTER_HI => self.program_counter[1],
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => self.read_prg_rom(addr),
      _ => {
        println!("Ignoring mem access at {:x}", addr);
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
      0x2000 => self.ppu.write_to_contol(data),
      0x2006 => self.ppu.write_to_address(data),
      0x2007 => self.ppu.write_to_data(data),

      0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & PPU_MIRROR_MASK;
        self.mem_write(mirror_down_addr, data);
      },
      PROGRAM_COUNTER_LO => self.program_counter[0] = data,
      PROGRAM_COUNTER_HI => self.program_counter[1] = data,
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => {
        panic!("Attempted to write to cartridge ROM space");
      }
      _ => {
        println!("Ignoring mem write at {:x}", addr);
      }
    }
  }
}

impl Bus {
  fn read_prg_rom(&self, mut addr: u16) -> u8 {
    addr -= 0x8000;

    // Mirror if needed (rom length is 16KB an )
    if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      addr = addr % 0x4000;
    }

    self.prg_rom[addr as usize]
  }
}
