use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

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
    registers: &mut [u8; 16],
    i: &mut u16,
    pc: &mut u16,
    screen: &mut [bool; 2048],
    stack: &mut Vec<u16>,
    event_pump: &mut sdl2::EventPump,
    c_mut: &Arc<Mutex<u8>>,
    keymap: &[&str; 16],
    invkeymap: &HashMap<&str, u8>,
) {
    match codes[0] {
        0x0 => {
            if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0 {
                *screen = [false; 2048];
            } else if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0xE {
                *pc = stack.pop().unwrap();
            }
        }
        0x1 => *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0x2 => {
            stack.push(*pc);
            *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]);
        }
        0x3 => {
            let x_val = registers[codes[1] as usize];
            let nn_val = ((codes[2] << 4) + (codes[3])) as u8;
            if x_val == nn_val {
                *pc += 2;
            }
        }
        0x4 => {
            let x_val = registers[codes[1] as usize];
            let nn_val = ((codes[2] << 4) + (codes[3])) as u8;
            if x_val != nn_val {
                *pc += 2;
            }
        }
        0x5 => {
            let x_val = registers[codes[1] as usize];
            let y_val = registers[codes[2] as usize];
            if x_val == y_val {
                *pc += 2;
            }
        }
        0x6 => registers[codes[1] as usize] = ((codes[2] << 4) + (codes[3])) as u8,
        0x7 => {
            registers[codes[1] as usize] =
                registers[codes[1] as usize].wrapping_add(((codes[2] << 4) + (codes[3])) as u8);
        }
        0x8 => match codes[3] {
            0x0 => registers[codes[1] as usize] = registers[codes[2] as usize],
            0x1 => registers[codes[1] as usize] |= registers[codes[2] as usize],
            0x2 => registers[codes[1] as usize] &= registers[codes[2] as usize],
            0x3 => registers[codes[1] as usize] ^= registers[codes[2] as usize],
            0x4 => {
                let old_val = registers[codes[1] as usize];
                registers[codes[1] as usize] =
                    registers[codes[1] as usize].wrapping_add(registers[codes[2] as usize]);
                if old_val > registers[codes[1] as usize] {
                    registers[0xF] = 1;
                }
            }
            0x5 => {
                registers[0xF] = 0;
                if registers[codes[1] as usize] < registers[codes[2] as usize] {
                    registers[0xF] = 1;
                }
                registers[codes[1] as usize] =
                    registers[codes[1] as usize].wrapping_sub(registers[codes[2] as usize]);
            }
            0x6 => {
                registers[0xF] = 0;
                if registers[codes[2] as usize] & 1 == 1 {
                    registers[0xF] = 1;
                }
                registers[codes[1] as usize] = registers[codes[2] as usize] >> 1;
            }
            0x7 => {
                registers[0xF] = 0;
                if registers[codes[2] as usize] < registers[codes[1] as usize] {
                    registers[0xF] = 1;
                }
                registers[codes[1] as usize] =
                    registers[codes[2] as usize] - registers[codes[1] as usize]
            }
            0xE => {
                registers[0xF] = 0;
                if ((registers[codes[2] as usize] & 64) >> 7) & 1 == 1 {
                    registers[0xF] = 1;
                }
                registers[codes[1] as usize] = registers[codes[2] as usize] << 1;
            }
            _ => {}
        },
        0x9 => {
            let x_val = registers[codes[1] as usize];
            let y_val = registers[codes[2] as usize];
            if x_val != y_val {
                *pc += 2;
            }
        }
        0xA => *i = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
        0xB => *pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]) + registers[0] as u16,
        0xC => {
            let mut rng = rand::thread_rng();
            let r_val = rng.gen_range(0..255);
            registers[codes[1] as usize] = (r_val & (codes[2] << 4) + (codes[3])) as u8;
        }
        0xD => {
            let mut x: usize = (registers[codes[1] as usize] % 64) as usize;
            let mut y: usize = (registers[codes[2] as usize] % 32) as usize;
            let mut i_val = *i;
            for _ in 0..codes[3] {
                let draw = memory[i_val as usize];
                let mut counter = 0;
                while x <= 63 && counter <= 7 {
                    if draw >> (7 - counter) & 1 == 1 {
                        let screen_val: usize = 64 * y + x;
                        if screen[screen_val] {
                            registers[0xF] = 1;
                        }
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
        //'X', '1', '2', '3', 'Q', 'W', 'E', 'A', 'S', 'D', 'Z', 'C', '4', 'R', 'F', 'V',
        0xE => {
            for event in (*event_pump).poll_iter() {
                match event {
                    Event::KeyDown { scancode, .. }
                        if scancode == Scancode::from_name(keymap[codes[1] as usize]) =>
                    {
                        if codes[2] == 0x9 && codes[3] == 0xE {
                            *pc += 2;
                            return;
                        } else if codes[2] == 0xA && codes[3] == 0x1 {
                            return;
                        }
                    }
                    _ => {}
                }
            }
            *pc += 2
        }
        0xF => {
            if codes[2] == 0x0 && codes[3] == 0x7 {
                let ocount = Arc::clone(&c_mut);
                let c = ocount.lock().unwrap();
                registers[codes[1] as usize] = *c;
            } else if codes[2] == 0x1 && codes[3] == 0x5 {
                let ocount = Arc::clone(&c_mut);
                let mut c = ocount.lock().unwrap();
                *c = registers[codes[1] as usize];
            } else if codes[2] == 0x1 && codes[3] == 0x8 {
            } else if codes[2] == 0x1 && codes[3] == 0xE {
                *i += registers[codes[1] as usize] as u16;
            } else if codes[2] == 0x0 && codes[3] == 0xA {
                for event in (*event_pump).poll_iter() {
                    match event {
                        Event::KeyDown { scancode, .. } => {
                            registers[codes[1] as usize] = invkeymap[scancode.unwrap().name()];
                            return;
                        }
                        _ => {}
                    }
                }
                *pc -= 2;
            } else if codes[2] == 0x2 && codes[3] == 0x9 {
                *i = (registers[codes[1] as usize] * 5) as u16;
            } else if codes[2] == 0x3 && codes[3] == 0x3 {
                let num = registers[codes[1] as usize] as u16;
                let dec: u16 = 10;

                for n in (1u16..4).rev() {
                    memory[(*i + (3 - n)) as usize] =
                        ((num % dec.pow(n.into())) / dec.pow((n - 1).into())) as u8;
                }
            } else if codes[2] == 0x5 && codes[3] == 0x5 {
                let lim = codes[1] + 1;
                for n in 0..lim {
                    memory[(*i + n) as usize] = registers[n as usize] as u8;
                }
            } else if codes[2] == 0x6 && codes[3] == 0x5 {
                let lim = codes[1] + 1;
                for n in 0..lim {
                    registers[n as usize] = memory[(*i + n) as usize];
                }
            }
        }
        _ => {}
    }
}

fn load_fonts(memory: &mut [u8; 4096]) {
    let font_data = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80,
    ]; // F
    for (i, x) in font_data.iter().enumerate() {
        memory[i] = *x;
    }
}

fn main() -> io::Result<()> {
    //window configurations
    const BOX_SIZE: u32 = 4;
    const PADDING: u32 = 50;
    const SHORT_SIDE: u32 = BOX_SIZE * 32 + PADDING * 2;
    const LONG_SIDE: u32 = BOX_SIZE * 64 + PADDING * 2;

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
    let mut registers: [u8; 16] = [0; 16];
    let mut f = File::open("tests/Chip-8 Demos/Sirpinski [Sergey Naydenov, 2010].ch8")?;
    let mut pc: u16 = 512;
    let mut screen: [bool; 2048] = [false; 2048];
    f.read(&mut memory[512..4096])?;

    //create keymappings
    let keymap: [&str; 16] = [
        "X", "1", "2", "3", "Q", "W", "E", "A", "S", "D", "Z", "C", "4", "R", "F", "V",
    ];
    let mut invkeymap = HashMap::new();
    invkeymap.insert("X", 0x0u8);
    invkeymap.insert("1", 0x1);
    invkeymap.insert("2", 0x2);
    invkeymap.insert("3", 0x3);
    invkeymap.insert("Q", 0x4);
    invkeymap.insert("W", 0x5);
    invkeymap.insert("E", 0x6);
    invkeymap.insert("A", 0x7);
    invkeymap.insert("S", 0x8);
    invkeymap.insert("D", 0x9);
    invkeymap.insert("Z", 0xA);
    invkeymap.insert("C", 0xB);
    invkeymap.insert("4", 0xC);
    invkeymap.insert("R", 0xD);
    invkeymap.insert("F", 0xE);
    invkeymap.insert("V", 0xF);

    load_fonts(&mut memory);

    let t_cps = 60;
    let t_frame_length = 1000000000 / t_cps;
    let c_mut = Arc::new(Mutex::new(60u8));
    let count = Arc::clone(&c_mut);

    thread::spawn(move || loop {
        let t_now = Instant::now();
        {
            let mut c = count.lock().unwrap();
            if *c != 0 {
                *c -= 1;
            }
        }
        while (t_now.elapsed().as_nanos()) < t_frame_length {}
    });

    let cps = 700;
    let frame_length = 1000000000 / cps;

    'running: loop {
        let now = Instant::now();
        let codes = fetch_decode(&mut memory, &mut pc);
        println!("{:?}", codes);
        execute(
            &codes,
            &mut memory,
            &mut registers,
            &mut i,
            &mut pc,
            &mut screen,
            &mut stack,
            &mut event_pump,
            &c_mut,
            &keymap,
            &invkeymap,
        );

        if codes[0] == 0xD {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for y in 0..32u32 {
                for x in 0..64u32 {
                    let screen_val: usize = (y * 64 + x).try_into().unwrap();
                    if screen[screen_val] {
                        let x_corner: i32 = (PADDING + BOX_SIZE * x).try_into().unwrap();
                        let y_corner: i32 = (PADDING + BOX_SIZE * y).try_into().unwrap();
                        canvas
                            .fill_rect(Rect::new(x_corner, y_corner, BOX_SIZE, BOX_SIZE))
                            .expect("Failure to draw");
                    }
                }
            }
        }
        let test = Scancode::from_name("1");
        //println!("{}", test == Scancode::Num1);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    scancode, keycode, ..
                } if scancode == Scancode::from_name("1") => {
                    println!("{:?}", keycode);
                    println!("{:?}", test);
                    println!("okay yeah what the fuck");
                    break 'running;
                }
                _ => {}
            }
        }
        canvas.present();
        while (now.elapsed().as_nanos()) < frame_length {}
    }
    Ok(())
}
