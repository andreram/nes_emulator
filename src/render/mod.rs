pub mod frame;
pub mod palette;

use crate::ppu::PPU;
use frame::Frame;

const TILE_SIZE_BYTES: u16 = 16;
const BACKROUND_SCREEN_WIDTH: usize = 32;

pub fn render(ppu: &PPU, frame: &mut Frame) {
  let bank = ppu.control.background_pattern_table_addr();

  // using first nametable for now
  for i in 0..=0x03c0 {
    let tile = ppu.vram[i] as u16;
    let tile_x = i % BACKROUND_SCREEN_WIDTH;
    let tile_y = i / BACKROUND_SCREEN_WIDTH;

    let tile = &ppu.chr_rom[(bank + tile * TILE_SIZE_BYTES) as usize..=(bank + tile * TILE_SIZE_BYTES + 15) as usize];

    for y in 0..=7 {
      let mut upper = tile[y];
      let mut lower = tile[y + 8];

      for x in (0..=7).rev() {
        let value = (upper & 1) << 1 | (lower & 1);
        upper = upper >> 1;
        lower = lower >> 1;
        let rgb = match value {
          0 => palette::SYSTEM_PALLETE[0x01],
          1 => palette::SYSTEM_PALLETE[0x23],
          2 => palette::SYSTEM_PALLETE[0x27],
          3 => palette::SYSTEM_PALLETE[0x30],
          _ => panic!("unreachable"),
        };

        frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
      }
    }
  }
}