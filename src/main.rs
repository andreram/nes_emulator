pub mod cpu;
pub mod ops;
pub mod bus;
pub mod rom;
pub mod trace;
pub mod ppu;
pub mod render;

use cpu::CPU;
use cpu::Mem;
use rom::Rom;
use rom::Mirroring;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use rand::Rng;
use trace::trace;
use ppu::PPU;
use render::frame::Frame;
use render::palette;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

const TILE_SIZE_BYTES: usize = 16;

fn main() {
  let scale_factor = 3.0;

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
    .window("Tile viewer", (Frame::WIDTH as f32 * scale_factor) as u32, (Frame::HEIGHT as f32 * scale_factor) as u32)
    .position_centered().build().unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  canvas.set_scale(scale_factor, scale_factor).unwrap();

  let creator = canvas.texture_creator();
  let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, Frame::WIDTH as u32, Frame::HEIGHT as u32).unwrap();

  fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    let user_input_cell = 0xff;

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          std::process::exit(0)
        },
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
          cpu.mem_write(user_input_cell, 0x77);
        },
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
          cpu.mem_write(user_input_cell, 0x73);
        },
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
          cpu.mem_write(user_input_cell, 0x61);
        },
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
          cpu.mem_write(user_input_cell, 0x64);
        },
        _ => {}
      }
    }
  }

  fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
 }

 fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
   let mut frame_idx = 0;
   let mut update = false;
   for i in 0x200..0x600u16 {
     let color_idx = cpu.mem_read(i);
     let (b1, b2, b3) = color(color_idx).rgb();
     if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
       frame[frame_idx] = b1;
       frame[frame_idx + 1] = b2;
       frame[frame_idx + 2] = b3;
       update = true;
     }
     frame_idx += 3;
   }
   update
 }

 fn show_tiles(chr_rom: &Vec<u8>, bank: usize) -> Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let mut tile_x = 0;
    let mut tile_y = 0; 
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..=255 {
      if tile_n != 0 && tile_n % 20 == 0 {
        tile_y += 10;
        tile_x = 0;
      }

      let tile = &chr_rom[(bank + tile_n * TILE_SIZE_BYTES)..=(bank + tile_n * TILE_SIZE_BYTES + 15)];

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

          frame.set_pixel(tile_x + x, tile_y + y, rgb);
        }
      }

      tile_x += 10;
    }

  frame
 }

  let raw_rom = std::fs::read("pacman.nes").unwrap();
  let rom = Rom::new(&raw_rom).unwrap();

  let mut frame = Frame::new();

  let mut cpu = CPU::new_with_gameloop(rom, move |ppu: &PPU| {
    render::render(ppu, &mut frame);
    texture.update(None, &frame.data, 256 * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => std::process::exit(0),
        _ => {},
      }
    }
  });

  cpu.reset();
  cpu.run();

  // let mut cpu = CPU::new(rom);
  // let ppu = PPU::new(vec![], Mirroring::VERTICAL);

  // cpu.reset();
  // cpu.program_counter = 0xC000;

  // let mut screen_state = [0u8; 32 * 3 * 32];
  // let mut rng = rand::thread_rng();

  // cpu.run_with_callback(move |cpu| {
  //   println!("{}", trace(cpu));
  //   // let rand_num_cell = 0xfe;

  //   // handle_user_input(cpu, &mut event_pump);
  //   // cpu.mem_write(rand_num_cell, rng.gen_range(1, 16));

  //   // if read_screen_state(cpu, &mut screen_state) {
  //   //   texture.update(None, &screen_state, 32 * 3).unwrap();
  //   //   canvas.copy(&texture, None, None).unwrap();
  //   //   canvas.present();
  //   // }
  // });

  // std::thread::sleep(std::time::Duration::new(0, 70000));
}
