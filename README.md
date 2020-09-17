# rip8
A CHIP-8 implementation with web based debugger built using [Rocket](https://rocket.rs/).

## Building
rip8 requires rust nightly as Rocket uses a lot of nightly features.

``` sh
rustup default nightly
```

Building is the usual with cargo.
``` sh
cargo build --release
```

## Usage

``` sh
USAGE:
    rip8 [FLAGS] [OPTIONS] <ROM_FILE>

FLAGS:
    -d, --debug      Enabled debugger window
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --clock-speed <HZ>    CPU clock speed (defaults to 500 Hz)

ARGS:
    <ROM_FILE>    Path to CHIP-8 ROM file (.ch8)USAGE:
    rip8 [FLAGS] [OPTIONS] <ROM_FILE>

FLAGS:
    -d, --debug      Enabled debugger window
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --clock-speed <HZ>    CPU clock speed (defaults to 500 Hz)

ARGS:
    <ROM_FILE>    Path to CHIP-8 ROM file (.ch8)
```

The debugger server is by default served at `localhost:8000`.
