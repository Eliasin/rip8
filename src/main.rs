mod mem;
mod logic;
mod exec;
mod io;

use exec::cpu::CPU;
use io::keys::SDL2Keyboard;

use sdl2::event::Event;

use std::fs::read;
use std::time::{ Instant, Duration };

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let sdl_context = sdl2::init()?;

    let mut event_pump = sdl_context.event_pump()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rip8", 800, 600).build()?;
    let mut canvas = window.into_canvas().build()?;

    let file_bytes = read("test_opcode.ch8")?;

    let mut cpu = CPU::new();
    cpu.map_program(file_bytes)?;

    const CPU_HZ: f64 = 500.0;
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

        let keyboard = SDL2Keyboard::new(event_pump.keyboard_state());
        cpu.execute_cycle(keyboard)?;
    };

    Ok(())

}
