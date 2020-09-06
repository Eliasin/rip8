use super::cpu::CPU;
use crate::io::keys::SDL2Keyboard;

use sdl2;
use sdl2::event::Event;

use std::time::{ Instant, Duration };

pub struct Runtime {
    cpu: CPU,
    sdl_context: sdl2::Sdl,
}

pub const CPU_HZ: f64 = 500.0;

impl Runtime {
    pub fn new(sdl_context: sdl2::Sdl) -> Runtime {
        Runtime{
            cpu: CPU::new(),
            sdl_context: sdl_context,
        }
    }

    pub fn start(&mut self, program: Vec<u8>) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut event_pump = self.sdl_context.event_pump()?;

        let video_subsystem = self.sdl_context.video()?;

        let window = video_subsystem.window("rip8", 800, 600).build()?;
        let mut canvas = window.into_canvas().build()?;

        self.cpu.map_program(program)?;

        let cpu_time_step: Duration = Duration::new(0, (1000000000.0 / CPU_HZ) as u32);

        let last_frame_time = Instant::now();

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
            match self.cpu.execute_cycle(keyboard_state) {
                Ok(_) => {
                    match instruction {
                        Ok(v) => println!("{}", v),
                        Err(error) => println!("{}", error),
                    }
                    println!("{}", self.cpu.inspect_register_file());
                },
                Err(error) => {
                    println!("{} \n {}", error, self.cpu.inspect_memory());
                    return Ok(());
                },
            }

            canvas.present();
        };

        Ok(())
    }
}
