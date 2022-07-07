pub mod frame;
pub mod palette;

use crate::ppu::PPU;
use frame::Frame;

const TILE_SIZE_BYTES: u16 = 16;
const BG_SCREEN_WIDTH_TILES: usize = 32;
const META_TILE_WIDTH_TILES: usize = 4;
const BG_SCREEN_WIDTH_META_TILES: usize = BG_SCREEN_WIDTH_TILES / META_TILE_WIDTH_TILES;

fn bg_pallette(ppu: &PPU, tile_row: usize, tile_column: usize) -> [u8; 4] {
  let attr_table_idx = tile_row / META_TILE_WIDTH_TILES * BG_SCREEN_WIDTH_META_TILES + tile_column / META_TILE_WIDTH_TILES;
  let attr_byte = ppu.vram[0x3c0 + attr_table_idx];

  let pallette_idx = match (tile_row % META_TILE_WIDTH_TILES / 2, tile_column % META_TILE_WIDTH_TILES / 2) {
    (0,0) => attr_byte & 0b11,
    (0,1) => (attr_byte >> 2) & 0b11,
    (1,0) => (attr_byte >> 4) & 0b11,
    (1,1) => (attr_byte >> 6) & 0b11,
    (_,_) => panic!("should be unreachable"),
  };

  let palette_start: usize = 1 + 4 * pallette_idx as usize;
  [ppu.palette_table[0], ppu.palette_table[palette_start], ppu.palette_table[palette_start + 1], ppu.palette_table[palette_start + 2]]
}

pub fn render(ppu: &PPU, frame: &mut Frame) {
  let bank = ppu.control.background_pattern_table_addr();

  // using first nametable for now
  for i in 0..=0x03c0 {
    let tile = ppu.vram[i] as u16;
    let tile_col = i % BG_SCREEN_WIDTH_TILES;
    let tile_row = i / BG_SCREEN_WIDTH_TILES;

    let tile = &ppu.chr_rom[(bank + tile * TILE_SIZE_BYTES) as usize..=(bank + tile * TILE_SIZE_BYTES + 15) as usize];
    let palette = bg_pallette(ppu, tile_row, tile_col);

    for y in 0..=7 {
      let mut hi = tile[y];
      let mut lo = tile[y + 8];

      for x in (0..=7).rev() {
        let value = (lo & 1) << 1 | (hi & 1);
        hi = hi >> 1;
        lo = lo >> 1;

        let rgb = match value {
          0 => palette::SYSTEM_PALLETTE[palette[0] as usize],
          1 => palette::SYSTEM_PALLETTE[palette[1] as usize],
          2 => palette::SYSTEM_PALLETTE[palette[2] as usize],
          3 => palette::SYSTEM_PALLETTE[palette[3] as usize],
          _ => panic!("unreachable"),
        };

        frame.set_pixel(tile_col * 8 + x, tile_row * 8 + y, rgb);
      }
    }
  }
}