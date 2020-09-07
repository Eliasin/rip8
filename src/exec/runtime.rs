use super::cpu::CPU;
use crate::io::keys::SDL2Keyboard;
use crate::io::screen::Screen;

use sdl2;
use sdl2::event::Event;

use std::time::{ Instant, Duration };

pub struct Runtime {
    cpu: CPU,
    sdl_context: sdl2::Sdl,
}

pub const CPU_HZ: f64 = 500.0;

fn draw_to_canvas(canvas: &mut sdl2::render::WindowCanvas, screen: &Screen) -> Result<(), Box<dyn std::error::Error>> {
    for (row, row_arr) in screen.inspect_screen().iter().enumerate() {
        for (column, pixel) in row_arr.iter().enumerate() {
            if *pixel {
                canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                canvas.draw_point(sdl2::rect::Point::new(column as i32, row as i32))?;
            } else {
                canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                canvas.draw_point(sdl2::rect::Point::new(column as i32, row as i32))?;
            }
        }
    }
    Ok(())
}

impl Runtime {
    pub fn new(sdl_context: sdl2::Sdl) -> Runtime {
        Runtime{
            cpu: CPU::new(),
            sdl_context,
        }
    }

    pub fn start(&mut self, program: Vec<u8>) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut event_pump = self.sdl_context.event_pump()?;

        let video_subsystem = self.sdl_context.video()?;

        let window = video_subsystem.window("rip8", 1280, 640).build()?;
        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(sdl2::pixels::Color::WHITE);
        canvas.clear();
        canvas.set_scale(20.0, 20.0)?;

        self.cpu.map_program(program)?;

        let cpu_time_step: Duration = Duration::new(0, (1000000000.0 / CPU_HZ) as u32);

        let mut last_frame_time = Instant::now();

        let mut screen = Screen::new();

        'running: loop {

            let next_frame_time = last_frame_time + cpu_time_step;
            if !(Instant::now() >= next_frame_time) {
                continue;
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => {
                        break 'running;
                    },
                    _ => {},
                };
            }

            let keyboard_state = SDL2Keyboard::new(event_pump.keyboard_state());
            let instruction = self.cpu.inspect_next_instruction();
            match self.cpu.execute_cycle(keyboard_state, &mut screen) {
                Ok(_) => {
                    match instruction {
                        Ok(v) => println!("{:?}", v),
                        Err(error) => println!("{}", error),
                    }
                    println!("{}", self.cpu.inspect_register_file());
                },
                Err(error) => {
                    println!("{} \n {}", error, self.cpu.inspect_memory());
                    return Ok(());
                },
            }

            if screen.has_changed() {
                draw_to_canvas(&mut canvas, &screen)?;
                screen.reset_changed();
                canvas.present();
            }

            last_frame_time = Instant::now();
        };

        Ok(())
    }
}
