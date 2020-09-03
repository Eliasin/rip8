mod mem;
mod logic;
mod exec;

use exec::cpu::CPU;

use std::fs::read;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let file_bytes = read("test_opcode.ch8")?;

    let mut cpu = CPU::new();
    cpu.map_program(file_bytes)?;

    for _ in 0..20 {
        match cpu.execute_cycle() {
            Ok(_) => {},
            Err(error) => println!("Error while executing: {}", error),
        }
    }

    Ok(())
}
