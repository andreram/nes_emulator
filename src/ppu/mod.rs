pub mod registers;

use crate::rom::Mirroring;
use registers::address::AddrRegister;
use registers::control::ControlRegister;
use registers::mask::MaskRegister;
use registers::scroll::ScrollRegister;
use registers::status::StatusRegister;

pub struct PPU {
  pub chr_rom: Vec<u8>,
  pub palette_table: [u8; 32],
  pub vram: [u8; 2048],
  pub oam_addr: u8,
  pub oam_data: [u8; 256],
  pub mirroring: Mirroring,

  address: AddrRegister,
  pub control: ControlRegister,
  mask: MaskRegister,
  scroll: ScrollRegister,
  status: StatusRegister,
  internal_data_buf: u8,
  cycles: usize,
  scanline: u16,
  nmi_interrupt: Option<bool>,
}

impl PPU {
  pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
    PPU {
      chr_rom: chr_rom,
      mirroring: mirroring,
      vram: [0; 2048],
      oam_addr: 0,
      oam_data: [0; 256],
      palette_table: [0; 32],
      internal_data_buf: 0,
      cycles: 0,
      scanline: 0,
      nmi_interrupt: None,

      address: AddrRegister::new(),
      control: ControlRegister::new(),
      mask: MaskRegister::new(),
      scroll: ScrollRegister::new(),
      status: StatusRegister::new(),
    }
  }

  pub fn new_empty_rom() -> Self {
    PPU::new(vec![0; 2048], Mirroring::HORIZONTAL)
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
      (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) | (Mirroring::HORIZONTAL, 3) => {
        index - 0x800
      }
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
      }
      0x2000..=0x3eff => {
        let result = self.internal_data_buf;
        self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      }
      // 0x3f20-0x3fff are mirrors of 0x3f00-0x3f1f
      0x3f00..=0x3fff => self.palette_table[(addr & 0x1f) as usize],
      _ => panic!("unexpected access to mirrored space {:x}", addr),
    }
  }

  pub fn write_to_ppu_addr(&mut self, value: u8) {
    self.address.update(value);
  }

  pub fn write_to_control(&mut self, value: u8) {
    let old_nmi_status = self.control.should_generate_vblank_nmi();
    self.control.update(value);
    if !old_nmi_status && self.control.should_generate_vblank_nmi() && self.status.in_vblank() {
      self.nmi_interrupt = Some(true);
    }
  }

  pub fn write_to_mask(&mut self, value: u8) {
    self.mask.update(value);
  }

  pub fn write_to_scroll(&mut self, value: u8) {
    self.scroll.update(value);
  }

  pub fn read_status(&mut self) -> u8 {
    let data = self.status.bits();

    self.status.set_vblank(false);
    self.address.reset_latch();
    self.scroll.reset_latch();
    data
  }

  pub fn write_to_data(&mut self, data: u8) {
    let addr = self.address.get();

    match addr {
      0..=0x1fff => println!("Attempted to write to CHR ROM space: {:x}", addr),
      0x2000..=0x3eff => self.vram[self.mirror_vram_addr(addr) as usize] = data,
      0x3f00..=0x3fff => self.palette_table[(addr & 0x1f) as usize] = data,
      _ => panic!("unexpected access to mirrored space {:x}", addr),
    };

    self.increment_vram_addr();
  }

  pub fn write_to_oam_addr(&mut self, data: u8) {
    self.oam_addr = data;
  }

  pub fn write_to_oam_data(&mut self, data: u8) {
    self.oam_data[self.oam_addr as usize] = data;
    self.oam_addr = self.oam_addr.wrapping_add(1);
  }

  pub fn read_oam_data(&self) -> u8 {
    self.oam_data[self.oam_addr as usize]
  }

  pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
    for i in data.iter() {
      self.write_to_oam_data(*i);
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    self.cycles += cycles as usize;
    if self.cycles >= 341 {
      self.cycles = self.cycles - 341;
      self.scanline += 1;

      if self.scanline == 241 {
        self.status.set_vblank(true);
        if self.control.should_generate_vblank_nmi() {
          self.nmi_interrupt = Some(true);
        }
      }

      if self.scanline >= 262 {
        self.scanline = 0;
        self.status.set_vblank(false);
      }
    }
  }

  pub fn poll_nmi_interrupt(&mut self) -> Option<bool> {
    self.nmi_interrupt.take()
  }

  pub fn background_pattern_table_addr(&self) -> u16 {
    self.control.background_pattern_table_addr()
  }

  pub fn nmi_interrupt_ready(&mut self) -> bool {
    self.nmi_interrupt.is_some()
  }
}

#[cfg(test)]
pub mod test {
  use super::*;

  #[test]
  fn test_ppu_writes() {
    let mut ppu = PPU::new_empty_rom();
    ppu.write_to_ppu_addr(0x23);
    ppu.write_to_ppu_addr(0x05);
    ppu.write_to_data(0x66);

    assert_eq!(ppu.vram[0x0305], 0x66);
  }

  #[test]
  fn test_ppu_vram_reads() {
    let mut ppu = PPU::new_empty_rom();
    ppu.vram[0x0305] = 0x66;

    ppu.write_to_ppu_addr(0x23);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.address.get(), 0x2306);
    assert_eq!(ppu.read_data(), 0x66);
  }

  #[test]
  fn test_ppu_reads_cross_page() {
    let mut ppu = PPU::new_empty_rom();
    ppu.vram[0x01ff] = 0x66;
    ppu.vram[0x0200] = 0x77;

    ppu.write_to_ppu_addr(0x21);
    ppu.write_to_ppu_addr(0xff);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x66);
    assert_eq!(ppu.read_data(), 0x77);
  }

  #[test]
  fn test_ppu_vram_reads_step_32() {
    let mut ppu = PPU::new_empty_rom();
    ppu.write_to_control(0b100);

    ppu.vram[0x01ff] = 0x66;
    ppu.vram[0x01ff + 32] = 0x77;
    ppu.vram[0x01ff + 64] = 0x88;
    ppu.vram[0x01ff + 65] = 0x99;

    println!("{}", ppu.vram[0x01ff + 65]);

    ppu.write_to_ppu_addr(0x21);
    ppu.write_to_ppu_addr(0xff);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x66);

    ppu.write_to_control(0);
    assert_eq!(ppu.read_data(), 0x77);
    assert_eq!(ppu.read_data(), 0x88);
    assert_eq!(ppu.read_data(), 0x99);
  }

  // Horizontal mirroring:
  //   [0x2000 A ] [0x2400 a ]
  //   [0x2800 B ] [0x2C00 b ]
  #[test]
  fn test_vram_horizontal_mirror() {
    let mut ppu = PPU::new_empty_rom();
    ppu.write_to_ppu_addr(0x24);
    ppu.write_to_ppu_addr(0x05);

    ppu.write_to_data(0x66); // write to a

    ppu.write_to_ppu_addr(0x28);
    ppu.write_to_ppu_addr(0x05);

    ppu.write_to_data(0x77); // write to B

    ppu.write_to_ppu_addr(0x20);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x66); // read from A

    ppu.write_to_ppu_addr(0x2c);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x77); // read from b
  }

  // Vertical mirroring:
  //   [0x2000 A ] [0x2400 B ]
  //   [0x2800 a ] [0x2C00 b ]
  #[test]
  fn test_vram_vertical_mirror() {
    let mut ppu = PPU::new(vec![0; 2048], Mirroring::VERTICAL);
    ppu.write_to_ppu_addr(0x20);
    ppu.write_to_ppu_addr(0x05);

    ppu.write_to_data(0x66); // write to A

    ppu.write_to_ppu_addr(0x2c);
    ppu.write_to_ppu_addr(0x05);

    ppu.write_to_data(0x77); // write to b

    ppu.write_to_ppu_addr(0x24);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x77); // read from B

    ppu.write_to_ppu_addr(0x28);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x66); // read from a
  }

  #[test]
  fn test_read_status_resets_latch() {
    let mut ppu = PPU::new_empty_rom();
    ppu.vram[0x0305] = 0x66;

    ppu.write_to_ppu_addr(0x21);
    ppu.write_to_ppu_addr(0x23);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_ne!(ppu.read_data(), 0x66);

    ppu.read_status();

    ppu.write_to_ppu_addr(0x23);
    ppu.write_to_ppu_addr(0x05);

    ppu.read_data(); // load buffer
    assert_eq!(ppu.read_data(), 0x66);
  }

  #[test]
  fn test_ppu_vram_mirroring() {
    let mut ppu = PPU::new_empty_rom();
    ppu.write_to_ppu_addr(0x63);
    ppu.write_to_ppu_addr(0x05);

    assert_eq!(ppu.address.get(), 0x2305);
  }

  #[test]
  fn test_read_status_resets_vblank() {
    let mut ppu = PPU::new_empty_rom();
    ppu.status.set_vblank(true);

    let status = ppu.read_status();

    assert_eq!(status >> 7, 1);
    assert_eq!(ppu.status.bits() >> 7, 0);
  }

  #[test]
  fn test_oam_read_write() {
    let mut ppu = PPU::new_empty_rom();
    ppu.write_to_oam_addr(0x10);
    ppu.write_to_oam_data(0x66);
    ppu.write_to_oam_data(0x77);

    ppu.write_to_oam_addr(0x10);
    assert_eq!(ppu.read_oam_data(), 0x66);

    ppu.write_to_oam_addr(0x11);
    assert_eq!(ppu.read_oam_data(), 0x77);
  }

  #[test]
  fn test_oam_dma() {
    let mut ppu = PPU::new_empty_rom();

    let mut data = [0x66; 256];
    data[0] = 0x77;
    data[255] = 0x88;

    ppu.write_to_oam_addr(0x10);
    ppu.write_oam_dma(&data);

    ppu.write_to_oam_addr(0x0f); // wrap around, 255 + 16
    assert_eq!(ppu.read_oam_data(), 0x88);

    ppu.write_to_oam_addr(0x10);
    assert_eq!(ppu.read_oam_data(), 0x77);

    ppu.write_to_oam_addr(0x11);
    assert_eq!(ppu.read_oam_data(), 0x66);
  }
}
