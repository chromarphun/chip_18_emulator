use std::io;
use std::io::prelude::*;
use std::fs::File;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;
use std::convert::TryInto;
 

fn decode_execute() {


}

fn main() -> io::Result<()> {
    //window configurations
    let BOX_SIZE: u32 = 4;
    let PADDING: u32 = 50;
    let SHORT_SIDE = BOX_SIZE * 32 + PADDING * 2;
    let LONG_SIDE = SHORT_SIDE * 2;

    //Initial graphics setup in SDL2
    let sdl_context=sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip 18 Emulator", LONG_SIDE, SHORT_SIDE)
    .position_centered()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    //create virtual hardware
    let mut i = 0;
    let mut stack = vec![0;0];
    let mut memory: [u8; 4096]=[0; 4096];
    let mut registers: [u8; 16]=[0; 16];
    let mut f = File::open("tests/exp")?;
    let mut pc = 0;
    let mut screen: [bool; 2048]=[false; 2048]; 
    screen[514]=true;
    f.read(&mut memory)?;

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for x in 0..63u32 {
            for y in 0..31u32 {
                let screen_val: usize = (x*32+y).try_into().unwrap();
                if screen[screen_val] {
                    //canvas.draw_point(Point::new(x,y));
                    let x_corner: i32=(PADDING+BOX_SIZE*x).try_into().unwrap();
                    let y_corner: i32=(PADDING+BOX_SIZE*y).try_into().unwrap();
                    canvas.fill_rect(Rect::new(x_corner, y_corner, BOX_SIZE, BOX_SIZE));
                }
            }
        }
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
