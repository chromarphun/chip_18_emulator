use sdl2::event::Event;

use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::convert::TryInto;

use std::io;

use std::time::Instant;

mod emulator;

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
    let mut screen = [false; 2048];
    let cps = 700;
    let frame_length = 1000000000 / cps;
    let mut em = emulator::Emulator::new();
    em.load_memory("tests/man_test.ch8");
    em.init();
    'running: loop {
        let now = Instant::now();
        let rc = em.next_frame(&mut event_pump, &mut screen);
        if rc {
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
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        canvas.present();
        while (now.elapsed().as_nanos()) < frame_length {}
    }
    Ok(())
}
