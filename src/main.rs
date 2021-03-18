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
fn fetch_decode(memory: &[u8; 4096], pc: &mut u16) -> [u16; 4] {
    let opcode1 = memory[*pc as usize] as u16;
    let opcode2 = memory[(*pc + 1) as usize] as u16;
    *pc += 2;
    [opcode1 >> 4, opcode1 & 15, opcode2 >> 4, opcode2 & 15]
}

fn execute(
    codes: &[u16; 4],
    memory: &mut [u8; 4096],
    registers: &mut [u16; 16],
    i: &mut u16,
    pc: &mut u16,
    screen: &mut [bool; 2048],
) {
    match codes[0] {
        0x0 => {
            if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0 {
                *screen = [false; 2048];
            } else if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0xE {
                *pc = stack.pop();
            }
        }
        0x1 => *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0x2 => {
            vec.push(*pc);
            *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]);
        }
        0x3 => {
            x_val = registers[codes[1] as usize];
            nn_val = (codes[2] << 4) + (codes[3]);
            if x_val == nn_val {
                *pc += 2;
            }
        }
        0x4 => {
            x_val = registers[codes[1] as usize];
            nn_val = (codes[2] << 4) + (codes[3]);
            if x_val != nn_val {
                *pc += 2;
            }
        }
        0x5 => {
            x_val = registers[codes[1] as usize];
            y_val = registers[codes[2] as usize];
            if x_val == y_val {
                *pc += 2;
            }
        }
        0x6 => registers[codes[1] as usize] = (codes[2] << 4) + (codes[3]),
        0x7 => {
            registers[codes[1] as usize] =
                registers[codes[1] as usize] + (codes[2] << 4) + (codes[3])
        }
        0x8 => match codes[3] {
            0x0 => registers[codes[1] as usize] = registers[code[2] as usize],
            0x1 => registers[codes[1] as usize] |= registers[code[2] as usize],
            0x2 => registers[codes[1] as usize] &= registers[code[2] as usize],
            0x3 => registers[codes[1] as usize] ^= registers[code[2] as usize],
        },
        0x9 => {
            x_val = registers[codes[1] as usize];
            y_val = registers[codes[2] as usize];
            if x_val != y_val {
                *pc += 2;
            }
        }
        0xA => *i = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0xD => {
            // println!("i is pointing at {}", memory[*i as usize]);
            let mut x = registers[codes[1] as usize] % 64;
            let mut y = registers[codes[2] as usize] % 32;
            let mut i_val = *i;
            for k in 0..codes[3] {
                let mut draw = memory[i_val as usize];
                let mut counter = 0;
                while x <= 63 && counter <= 7 {
                    if draw >> (7 - counter) & 1 == 1 {
                        let screen_val: usize = (64 * y + x).try_into().unwrap();
                        screen[screen_val] = !screen[screen_val];
                    }
                    x += 1;
                    counter += 1;
                }
                i_val += 1;
                x -= counter;
                y += 1;
            }
        }
        _ => {}
    }
}

fn main() -> io::Result<()> {
    //window configurations
    let BOX_SIZE: u32 = 4;
    let PADDING: u32 = 50;
    let SHORT_SIDE = BOX_SIZE * 32 + PADDING * 2;
    let LONG_SIDE = BOX_SIZE * 64 + PADDING * 2;

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
    let mut i: u16 = 0;
    let mut stack = vec![0; 0];
    let mut memory: [u8; 4096] = [0; 4096];
    let mut registers: [u16; 16] = [0; 16];
    let mut f = File::open("tests/IBM Logo.ch8")?;
    let mut pc: u16 = 0;
    let mut screen: [bool; 2048] = [false; 2048];
    f.read(&mut memory[512..4096])?;
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let codes = fetch_decode(&mut memory, &mut pc);
        // println!("{:?}", codes);
        // println!("pc: {}", pc);
        execute(
            &codes,
            &mut memory,
            &mut registers,
            &mut i,
            &mut pc,
            &mut screen,
        );
        for y in 0..32u32 {
            for x in 0..64u32 {
                let screen_val: usize = (y * 64 + x).try_into().unwrap();
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
