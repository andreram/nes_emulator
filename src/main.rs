pub mod cpu;
pub mod ops;
pub mod bus;
pub mod rom;
pub mod trace;
pub mod ppu;
pub mod render;
pub mod joypad;

use cpu::CPU;
use rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use ppu::PPU;
use render::frame::Frame;
use joypad::Joypad;
use joypad::JoypadButton;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

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

  let raw_rom = std::fs::read("pacman.nes").unwrap();
  let rom = Rom::new(&raw_rom).unwrap();

  let mut frame = Frame::new();

  let mut key_map = HashMap::new();
  key_map.insert(Keycode::Down, JoypadButton::DOWN);
  key_map.insert(Keycode::Up, JoypadButton::UP);
  key_map.insert(Keycode::Right, JoypadButton::RIGHT);
  key_map.insert(Keycode::Left, JoypadButton::LEFT);
  key_map.insert(Keycode::Space, JoypadButton::SELECT);
  key_map.insert(Keycode::Return, JoypadButton::START);
  key_map.insert(Keycode::A, JoypadButton::BUTTON_A);
  key_map.insert(Keycode::S, JoypadButton::BUTTON_B);

  let mut cpu = CPU::new_with_gameloop(rom, move |ppu: &PPU, joypad: &mut Joypad| {
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

        Event::KeyDown { keycode, .. } => {
          if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
            joypad.set_button_pressed(*key, true);
          }
        },

        Event::KeyUp { keycode, .. } => {
          if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
            joypad.set_button_pressed(*key, false);
          }
        },
        _ => {},
      }
    }
  });

  cpu.reset();
  cpu.run();
}
