mod mem;
mod logic;
mod exec;
mod io;

use exec::runtime::Runtime;

use clap::{ App, Arg };

use std::fs::read;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let matches = App::new("rip8")
        .version("1.0")
        .author("Steven Pham")
        .about("CHIP-8 Emulator written in rust")
        .arg(Arg::with_name("rom")
             .help("Path to CHIP-8 ROM file (.ch8)")
             .value_name("ROM_FILE")
             .takes_value(true)
             .required(true)
        ).arg(Arg::with_name("clock-speed")
              .short("c")
              .long("clock-speed")
              .help("CPU clock speed (defaults to 500 Hz)")
              .value_name("HZ")
              .takes_value(true)
        ).get_matches();

    let rom_path = matches.value_of("rom").unwrap();
    let clock_speed: f64 = match matches.value_of("clock-speed").unwrap_or("500.0").parse() {
        Ok(v) => v,
        Err(error) => panic!("Error while parsing clock speed: {}", error),
    };

    let sdl_context = sdl2::init()?;

    let file_bytes = read(rom_path)?;

    let mut runtime = Runtime::new(sdl_context);

    runtime.start(file_bytes, clock_speed)?;

    Ok(())

}
