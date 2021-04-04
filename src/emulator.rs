use rand::Rng;
use sdl2::event::Event;

use sdl2::keyboard::Scancode;

use std::collections::HashMap;
use std::fs::File;

use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    pc: u16,
    stack: Vec<u16>,
    c_mut: Arc<Mutex<u8>>,
    keymap: [String; 16],
    invkeymap: HashMap<String, u8>,
    init: bool,
    draw: bool,
}

impl Emulator {
    pub fn new() -> Emulator {
        let keymap: [String; 16] = [
            "X".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "Q".to_string(),
            "W".to_string(),
            "E".to_string(),
            "A".to_string(),
            "S".to_string(),
            "D".to_string(),
            "Z".to_string(),
            "C".to_string(),
            "4".to_string(),
            "R".to_string(),
            "F".to_string(),
            "V".to_string(),
        ];
        let mut invkeymap = HashMap::new();
        invkeymap.insert("X".to_string(), 0x0u8);
        invkeymap.insert("1".to_string(), 0x1);
        invkeymap.insert("2".to_string(), 0x2);
        invkeymap.insert("3".to_string(), 0x3);
        invkeymap.insert("Q".to_string(), 0x4);
        invkeymap.insert("W".to_string(), 0x5);
        invkeymap.insert("E".to_string(), 0x6);
        invkeymap.insert("A".to_string(), 0x7);
        invkeymap.insert("S".to_string(), 0x8);
        invkeymap.insert("D".to_string(), 0x9);
        invkeymap.insert("Z".to_string(), 0xA);
        invkeymap.insert("C".to_string(), 0xB);
        invkeymap.insert("4".to_string(), 0xC);
        invkeymap.insert("R".to_string(), 0xD);
        invkeymap.insert("F".to_string(), 0xE);
        invkeymap.insert("V".to_string(), 0xF);
        let i: u16 = 0;
        let stack = vec![0; 0];
        let mut memory: [u8; 4096] = [0; 4096];
        let registers: [u8; 16] = [0; 16];
        let pc: u16 = 512;
        let c_mut = Arc::new(Mutex::new(60u8));
        let init = false;
        let draw = false;

        Self::load_fonts(&mut memory);
        Emulator {
            memory,
            registers,
            i,
            pc,
            stack,
            c_mut,
            keymap,
            invkeymap,
            init,
            draw,
        }
    }
    pub fn load_memory(&mut self, path: &str) {
        let mut f = File::open(path).expect("File problem!");
        f.read(&mut self.memory[512..4096]).expect("Read issue!");
    }
    pub fn init(&mut self) {
        let t_cps = 60;
        let t_frame_length = 1000000000 / t_cps;
        let count = Arc::clone(&self.c_mut);
        self.init = true;
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
    }

    pub fn next_frame(
        &mut self,
        event_pump: &mut sdl2::EventPump,
        screen: &mut [bool; 2048],
    ) -> bool {
        let codes = self.fetch_decode();
        println!(
            "Code: {:x}{:x}{:x}{:x}, PC: {:x}, (From file: {:x})",
            codes[0],
            codes[1],
            codes[2],
            codes[3],
            self.pc - 2,
            self.pc - 514
        );
        self.execute(codes, event_pump, screen);
        println!(
            "PC: {:x}, registers: {:?}, i: {:x}\n",
            self.pc, self.registers, self.i
        );

        self.draw
    }

    fn fetch_decode(&mut self) -> [u16; 4] {
        let opcode1 = self.memory[self.pc as usize] as u16;
        let opcode2 = self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        [opcode1 >> 4, opcode1 & 15, opcode2 >> 4, opcode2 & 15]
    }

    fn execute(
        &mut self,
        codes: [u16; 4],
        event_pump: &mut sdl2::EventPump,
        screen: &mut [bool; 2048],
    ) {
        self.draw = false;
        match codes[0] {
            0x0 => {
                if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0 {
                    *screen = [false; 2048];
                } else if codes[1] == 0 && codes[2] == 0xE && codes[3] == 0xE {
                    self.pc = self.stack.pop().unwrap();
                }
            }
            0x1 => self.pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
            0x2 => {
                self.stack.push(self.pc);
                self.pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]);
            }
            0x3 => {
                let x_val = self.registers[codes[1] as usize];
                let nn_val = ((codes[2] << 4) + (codes[3])) as u8;
                if x_val == nn_val {
                    self.pc += 2;
                }
            }
            0x4 => {
                let x_val = self.registers[codes[1] as usize];
                let nn_val = ((codes[2] << 4) + (codes[3])) as u8;
                if x_val != nn_val {
                    self.pc += 2;
                }
            }
            0x5 => {
                let x_val = self.registers[codes[1] as usize];
                let y_val = self.registers[codes[2] as usize];
                if x_val == y_val {
                    self.pc += 2;
                }
            }
            0x6 => self.registers[codes[1] as usize] = ((codes[2] << 4) + (codes[3])) as u8,
            0x7 => {
                self.registers[codes[1] as usize] = self.registers[codes[1] as usize]
                    .wrapping_add(((codes[2] << 4) + (codes[3])) as u8);
            }
            0x8 => match codes[3] {
                0x0 => self.registers[codes[1] as usize] = self.registers[codes[2] as usize],
                0x1 => self.registers[codes[1] as usize] |= self.registers[codes[2] as usize],
                0x2 => self.registers[codes[1] as usize] &= self.registers[codes[2] as usize],
                0x3 => self.registers[codes[1] as usize] ^= self.registers[codes[2] as usize],
                0x4 => {
                    let old_val = self.registers[codes[1] as usize];
                    self.registers[0xF] = 0;
                    self.registers[codes[1] as usize] = self.registers[codes[1] as usize]
                        .wrapping_add(self.registers[codes[2] as usize]);
                    if old_val > self.registers[codes[1] as usize] {
                        self.registers[0xF] = 1;
                    }
                }
                0x5 => {
                    self.registers[0xF] = 0;
                    if self.registers[codes[1] as usize] > self.registers[codes[2] as usize] {
                        self.registers[0xF] = 1;
                    }
                    self.registers[codes[1] as usize] = self.registers[codes[1] as usize]
                        .wrapping_sub(self.registers[codes[2] as usize]);
                }
                0x6 => {
                    self.registers[0xF] = 0;
                    if self.registers[codes[2] as usize] & 1 == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.registers[codes[1] as usize] = self.registers[codes[2] as usize] >> 1;
                }
                0x7 => {
                    self.registers[0xF] = 0;
                    if self.registers[codes[2] as usize] > self.registers[codes[1] as usize] {
                        self.registers[0xF] = 1;
                    }
                    self.registers[codes[1] as usize] = self.registers[codes[2] as usize]
                        .wrapping_sub(self.registers[codes[1] as usize]);
                }
                0xE => {
                    self.registers[0xF] = 0;
                    if ((self.registers[codes[2] as usize]) >> 7) & 1 == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.registers[codes[1] as usize] = self.registers[codes[2] as usize] << 1;
                }
                _ => {}
            },
            0x9 => {
                let x_val = self.registers[codes[1] as usize];
                let y_val = self.registers[codes[2] as usize];
                if x_val != y_val {
                    self.pc += 2;
                }
            }
            0xA => self.i = (codes[1] << 8) + (codes[2] << 4) + (codes[3]),
            0xB => {
                self.pc = (codes[1] << 8) + (codes[2] << 4) + (codes[3]) + self.registers[0] as u16
            }
            0xC => {
                let mut rng = rand::thread_rng();
                let r_val = rng.gen_range(0..255);
                self.registers[codes[1] as usize] = (r_val & ((codes[2] << 4) + (codes[3]))) as u8;
            }
            0xD => {
                self.draw = true;
                let mut x: usize = (self.registers[codes[1] as usize] % 64) as usize;
                let mut y: usize = (self.registers[codes[2] as usize] % 32) as usize;
                let mut i_val = self.i;
                self.registers[0xF] = 0;
                for _ in 0..codes[3] {
                    let draw = self.memory[i_val as usize];
                    let mut counter = 0;
                    while x <= 63 && counter <= 7 && y <= 31 {
                        if ((draw >> (7 - counter)) & 1) == 1 {
                            let screen_val: usize = 64 * y + x;
                            if screen[screen_val] {
                                self.registers[0xF] = 1;
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
                            if scancode
                                == Scancode::from_name(
                                    &self.keymap[self.registers[codes[1] as usize] as usize],
                                ) =>
                        {
                            if codes[2] == 0x9 && codes[3] == 0xE {
                                self.pc += 2;
                                return;
                            } else if codes[2] == 0xA && codes[3] == 0x1 {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                self.pc += 2
            }
            0xF => {
                if codes[2] == 0x0 && codes[3] == 0x7 {
                    let ocount = Arc::clone(&self.c_mut);
                    let c = ocount.lock().unwrap();
                    self.registers[codes[1] as usize] = *c;
                } else if codes[2] == 0x1 && codes[3] == 0x5 {
                    let ocount = Arc::clone(&self.c_mut);
                    let mut c = ocount.lock().unwrap();
                    *c = self.registers[codes[1] as usize];
                } else if codes[2] == 0x1 && codes[3] == 0x8 {
                } else if codes[2] == 0x1 && codes[3] == 0xE {
                    self.i += self.registers[codes[1] as usize] as u16;
                } else if codes[2] == 0x0 && codes[3] == 0xA {
                    for event in (*event_pump).poll_iter() {
                        match event {
                            Event::KeyDown { scancode, .. } => {
                                self.registers[codes[1] as usize] =
                                    self.invkeymap[scancode.unwrap().name()];
                                return;
                            }
                            _ => {}
                        }
                    }
                    self.pc -= 2;
                } else if codes[2] == 0x2 && codes[3] == 0x9 {
                    self.i = (self.registers[codes[1] as usize] * 5) as u16;
                } else if codes[2] == 0x3 && codes[3] == 0x3 {
                    let num = self.registers[codes[1] as usize] as u16;
                    let dec: u16 = 10;

                    for n in (1u16..4).rev() {
                        self.memory[(self.i + (3 - n)) as usize] =
                            ((num % dec.pow(n.into())) / dec.pow((n - 1).into())) as u8;
                    }
                } else if codes[2] == 0x5 && codes[3] == 0x5 {
                    let lim = codes[1] + 1;
                    for n in 0..lim {
                        self.memory[(self.i + n) as usize] = self.registers[n as usize] as u8;
                    }
                } else if codes[2] == 0x6 && codes[3] == 0x5 {
                    let lim = codes[1] + 1;
                    for n in 0..lim {
                        self.registers[n as usize] = self.memory[(self.i + n) as usize];
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
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn op_code_test() {
        let mut em = Emulator::new();
        em.registers[0] = 0;
        em.registers[1] = 0;
        em.memory[512] = 0xf4;
        em.memory[513] = 0x65;
        em.i = 0x400;
        em.memory[0x400] = 0x15;
        em.memory[0x401] = 0x16;
        em.memory[0x402] = 0x17;
        em.memory[0x403] = 0x18;
        em.memory[0x404] = 0x19;
        em.memory[0x405] = 0x20;
        let mut screen = [false; 2048];
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        em.next_frame(&mut event_pump, &mut screen);
        assert_eq!(em.registers[4], 0x19);
        assert_eq!(em.registers[5], 0x0);
    }
}
