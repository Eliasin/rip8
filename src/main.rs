mod mem;
mod logic;
mod exec;
mod io;

use exec::runtime::Runtime;

use std::fs::read;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let sdl_context = sdl2::init()?;

    let file_bytes = read("test_opcode.ch8")?;

    let mut runtime = Runtime::new(sdl_context);

    runtime.start(file_bytes)?;

    Ok(())

}
