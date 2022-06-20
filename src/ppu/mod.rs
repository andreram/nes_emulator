pub mod registers;

use crate::rom::Mirroring;
use registers::address::AddrRegister;
use registers::control::ControlRegister;

pub struct PPU {
  pub chr_rom: Vec<u8>,
  pub palette_table: [u8; 32],
  pub vram: [u8; 2048],
  pub oam_data: [u8; 256],
  pub mirroring: Mirroring,

  address: AddrRegister,
  control: ControlRegister,
  internal_data_buf: u8,
}

impl PPU {
  pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
    PPU {
      chr_rom: chr_rom,
      mirroring: mirroring,
      vram: [0; 2048],
      oam_data: [0; 256],
      palette_table: [0; 32],
      internal_data_buf: 0,

      address: AddrRegister::new(),
      control: ControlRegister::new(),
    }
  }

  fn increment_vram_addr(&mut self) {
    self.address.increment(self.control.vram_addr_increment());
  }

  fn mirror_vram_addr(&self, addr: u16) -> u16 {
    // mirror down 0x3000-0x3eff to 0x2000-0x2eff
    let mirrored_addr = addr & 0x2fff;
    let index = mirrored_addr - 0x2000;
    let nametable = index / 0x400;

    match (&self.mirroring, nametable) {
      (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) | (Mirroring::HORIZONTAL, 3) => index - 0x800,
      (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => index - 0x400,
      _ => index,
    }
  }

  pub fn read_data(&mut self) -> u8 {
    let addr = self.address.get();
    self.increment_vram_addr();

    match addr {
      0..=0x1fff => {
        let result = self.internal_data_buf;
        self.internal_data_buf = self.chr_rom[addr as usize];
        result
      },
      0x2000..=0x3eff => {
        let result = self.internal_data_buf;
        self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      },
      // 0x3f20-0x3fff are mirrors of 0x3f00-0x3f1f
      0x3f00..=0x3fff => self.palette_table[(addr & 0x1f) as usize],
      _ => panic!("unexpected access to mirrored space {:x}", addr),
    }
  }

  pub fn write_to_address(&mut self, value: u8) {
    self.address.update(value);
  }

  pub fn write_to_contol(&mut self, value: u8) {
    self.control.update(value);
  }

  pub fn write_to_data(&mut self, data: u8) {
    let addr = self.address.get();

    match addr {
      0..=0x1fff => panic!("Attempted to write to CHR ROM space: {:x}", addr),
      0x2000..=0x3eff => self.vram[self.mirror_vram_addr(addr) as usize] = data,
      0x3f00..=0x3fff => self.palette_table[(addr & 0x1f) as usize] = data,
      _ => panic!("unexpected access to mirrored space {:x}", addr),
    };

    self.increment_vram_addr();
  }
}
