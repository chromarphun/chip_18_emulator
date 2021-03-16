use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::time::Duration;

// extract the four nibbles for the two byte opcode
fn decode(opcode1: u8, opcode2: u8) -> [u8; 4] {
    [opcode1 >> 4, opcode1 & 15, opcode2 >> 4, opcode2 & 15]
}

fn execute(
    codes: &[u8; 4],
    memory: &mut [u8; 4096],
    registers: &mut [u8; 16],
    i: &mut u8,
    screen: &mut [bool; 2048],
) {
    match codes[0] {
        0x0 => {
            *memory = [0; 4096];
        }
        0x1 => *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0x6 => registers[codes[1]] = (codes[2] << 4) + (codes[3]),
        0x7 => registers[codes[1]] = registers[codes[1]] + (codes[2] << 4) + (codes[3]),
        0xA => *i = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0xD => {
            let x = registers[code[1]] % 63;
            let y = registers[code[2]] % 31;
            let draw = memory[*i];
            for k in 0..code[3] {
                let counter = 0;
                while x <= 63 {}
            }
        }
    }
}

fn main() -> io::Result<()> {
    //window configurations
    let BOX_SIZE: u32 = 4;
    let PADDING: u32 = 50;
    let SHORT_SIDE = BOX_SIZE * 32 + PADDING * 2;
    let LONG_SIDE = SHORT_SIDE * 2;

    //Initial graphics setup in SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip 18 Emulator", LONG_SIDE, SHORT_SIDE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    //create virtual hardware
    let mut i: u8 = 0;
    let mut stack = vec![0; 0];
    let mut memory: [u8; 4096] = [0; 4096];
    let mut registers: [u8; 16] = [0; 16];
    let mut f = File::open("tests/exp")?;
    let mut pc: u8 = 0;
    let mut screen: [bool; 2048] = [false; 2048];
    f.read(&mut memory)?;
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for x in 0..63u32 {
            for y in 0..31u32 {
                let screen_val: usize = (x * 32 + y).try_into().unwrap();
                if screen[screen_val] {
                    let x_corner: i32 = (PADDING + BOX_SIZE * x).try_into().unwrap();
                    let y_corner: i32 = (PADDING + BOX_SIZE * y).try_into().unwrap();
                    canvas.fill_rect(Rect::new(x_corner, y_corner, BOX_SIZE, BOX_SIZE));
                }
            }
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
