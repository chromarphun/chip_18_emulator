use std::io;
use std::io::prelude::*;
use std::fs::File;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
 


fn main() -> io::Result<()> {
    //Initial graphics setup in SDL2
    let sdl_context=sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip 18 Emulator", 800, 600)
    .position_centered()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    //create virtual hardware
    let mut i = 0;
    let mut stack = Vec::new();
    let mut memory: [u8; 4096]=[0; 4096];
    let mut registers: [u8; 16]=[0; 16];
    let mut f = File::open("tests/exp")?;
    let mut pc = 0;
    f.read(&mut memory)?;
    
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
