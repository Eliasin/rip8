mod mem;
mod logic;
mod exec;
mod io;

use exec::cpu::CPU;
use io::keys::SDL2Keyboard;

use std::fs::read;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let sdl_context = sdl2::init()?;

    let event_pump = sdl_context.event_pump()?;
    let keyboard = SDL2Keyboard::new(&event_pump);

    let file_bytes = read("test_opcode.ch8")?;

    let mut cpu = CPU::new(&keyboard);
    cpu.map_program(file_bytes)?;

    for _ in 0..20 {
        match cpu.execute_cycle() {
            Ok(_) => {},
            Err(error) => println!("Error while executing: {}", error),
        }
    }

    Ok(())
}
