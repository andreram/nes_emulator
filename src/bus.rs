use crate::cpu::Mem;
use crate::rom::Rom;
use crate::ppu::PPU;

pub struct Bus<'call> {
  cpu_vram: [u8; 2048],
  prg_rom: Vec<u8>,
  // TODO: Remove this
  // program_counter: [u8; 2],
  ppu: PPU,
  cycles: usize,
  gameloop_callback: Box<dyn FnMut(&PPU) + 'call>,
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PRG_ROM_MAP: u16 = 0x8000;
const PRG_ROM_MAP_END: u16 = 0xFFFF;
const RAM_MIRROR_MASK: u16 = 0b0000_0111_1111_1111; // 0x0 - 0x7FF
const PPU_MIRROR_MASK: u16 = 0b0010_0000_0000_0111; // 0x2000 - 0x2007
// const PROGRAM_COUNTER_LO: u16 = 0xFFFC;
// const PROGRAM_COUNTER_HI: u16 = 0xFFFD;

impl<'a> Mem for Bus<'a> {
  fn mem_read(&mut self, addr: u16) -> u8 {
    match addr {
      RAM ..= RAM_MIRRORS_END => {
        let mirror_down_addr = addr & RAM_MIRROR_MASK;
        self.cpu_vram[mirror_down_addr as usize]
      },
      0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
        // panic!("Attempted to read from write-only PPU address {:x}", addr);
        0
      },
      0x2002 => self.ppu.read_status(),
      0x2004 => self.ppu.read_oam_data(),
      0x2007 => self.ppu.read_data(),

      0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & PPU_MIRROR_MASK;
        self.mem_read(mirror_down_addr)
      },
      // PROGRAM_COUNTER_LO => self.program_counter[0],
      // PROGRAM_COUNTER_HI => self.program_counter[1],
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => self.read_prg_rom(addr),
      _ => {
        // println!("Ignoring mem access at {:x}", addr);
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
      0x2000 => self.ppu.write_to_control(data),
      0x2001 => self.ppu.write_to_mask(data),
      0x2003 => self.ppu.write_to_oam_addr(data),
      0x2004 => self.ppu.write_to_oam_data(data),
      0x2005 => self.ppu.write_to_scroll(data),
      0x2006 => self.ppu.write_to_ppu_addr(data),
      0x2007 => self.ppu.write_to_data(data),

      0x2002 => panic!("Attempted to write to PPU status register"),

      0x4014 => {
        let mut buf: [u8; 256] = [0; 256];
        for i in 0..=0xFF {
          let hi = (data as u16) << 8;
          buf[i] = self.mem_read(hi + i as u16);
        }

        self.ppu.write_oam_dma(&buf);
      }

      0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & PPU_MIRROR_MASK;
        self.mem_write(mirror_down_addr, data);
      },
      // PROGRAM_COUNTER_LO => self.program_counter[0] = data,
      // PROGRAM_COUNTER_HI => self.program_counter[1] = data,
      PRG_ROM_MAP ..= PRG_ROM_MAP_END => {
        panic!("Attempted to write to cartridge ROM space");
      }
      _ => {
        // println!("Ignoring mem write at {:x}", addr);
      }
    }
  }
}

impl<'a> Bus<'a> {
  pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
  where
    F: FnMut(&PPU) + 'call,
  {
    Bus {
      cpu_vram: [0; 2048],
      prg_rom: rom.prg_rom,
      // program_counter: [0x0, 0x86],
      ppu: PPU::new(rom.chr_rom, rom.screen_mirroring),
      cycles: 0,
      gameloop_callback: Box::from(gameloop_callback),
    }
  }

  fn read_prg_rom(&self, mut addr: u16) -> u8 {
    addr -= 0x8000;

    // Mirror if needed (rom length is 16KB an )
    if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      addr = addr % 0x4000;
    }

    self.prg_rom[addr as usize]
  }

  pub fn tick(&mut self, cycles: u8) {
    self.cycles += cycles as usize;

    let nmi_before = self.ppu.nmi_interrupt_ready();
    self.ppu.tick(cycles * 3);
    let nmi_after = self.ppu.nmi_interrupt_ready();

    if !nmi_before && nmi_after {
      (self.gameloop_callback)(&self.ppu);
    }
  }

  pub fn poll_nmi_interrupt(&mut self) -> Option<bool> {
    self.ppu.poll_nmi_interrupt()
  }
}