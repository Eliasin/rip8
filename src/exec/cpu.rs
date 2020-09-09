use super::executor;
use crate::io::keys::Keyboard;
use crate::io::screen::Screen;
use crate::logic::decoder;
use crate::logic::instruction::Instruction;
use crate::mem::register::RegisterFile;
use crate::mem::{RAM, RAM_SIZE};

use std::error::Error;

pub const RAM_PROG_START: usize = 0x200;
pub const RAM_DIGIT_SPRITE_START: usize = 0xFF;
pub const DIGIT_SPRITE_SIZE: usize = 5;

#[derive(Debug)]
pub struct RAMOutOfBoundsError {
    msg: &'static str,
}

impl std::fmt::Display for RAMOutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for RAMOutOfBoundsError {}

impl RAMOutOfBoundsError {
    pub fn new() -> RAMOutOfBoundsError {
        RAMOutOfBoundsError {
            msg: "Attempted access to out of bounds region of RAM",
        }
    }
}

pub struct CPU {
    register_file: RegisterFile,
    ram: RAM,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            register_file: RegisterFile::new(),
            ram: [0; RAM_SIZE],
        };

        cpu.register_file.PC = RAM_PROG_START as u16;

        cpu
    }

    fn map_data(&mut self, data: Vec<u8>, start: usize) -> Result<(), RAMOutOfBoundsError> {
        let data_length = data.len();
        if data_length > self.ram.len() - start {
            return Err(RAMOutOfBoundsError::new());
        }

        let program_area = &mut self.ram[start..start + data_length];
        program_area.copy_from_slice(data.as_slice());

        Ok(())
    }

    pub fn map_program(&mut self, program: Vec<u8>) -> Result<(), RAMOutOfBoundsError> {
        self.map_data(program, RAM_PROG_START)
    }

    pub fn map_digit_sprites(&mut self) {
        let zero = vec![0xF0, 0x90, 0x90, 0x90, 0xF0];
        let one = vec![0x20, 0x60, 0x20, 0x20, 0x70];
        let two = vec![0xF0, 0x10, 0xF0, 0x80, 0xF0];
        let three = vec![0xF0, 0x10, 0xF0, 0x10, 0xF0];
        let four = vec![0x90, 0x90, 0xF0, 0x10, 0x10];
        let five = vec![0xF0, 0x80, 0xF0, 0x10, 0xF0];
        let six = vec![0xF0, 0x80, 0xF0, 0x90, 0xF0];
        let seven = vec![0xF0, 0x10, 0x20, 0x40, 0x40];
        let eight = vec![0xF0, 0x90, 0xF0, 0x90, 0xF0];
        let nine = vec![0xF0, 0x90, 0xF0, 0x10, 0xF0];
        let a = vec![0xF0, 0x90, 0xF0, 0x90, 0x90];
        let b = vec![0xE0, 0x90, 0xE0, 0x90, 0xE0];
        let c = vec![0xF0, 0x80, 0x80, 0x80, 0xF0];
        let d = vec![0xE0, 0x90, 0x90, 0x90, 0xE0];
        let e = vec![0xF0, 0x80, 0xF0, 0x80, 0xF0];
        let f = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let digits = vec![zero, one, two, three, four, five, six, seven, eight, nine ,a, b, c, d, e, f];
        for (i, digit) in digits.into_iter().enumerate() {
            self.map_data(digit, RAM_DIGIT_SPRITE_START + (i * DIGIT_SPRITE_SIZE)).unwrap();
        }
    }

    fn get_next_instruction_bytes(&self) -> Result<(u8, u8), RAMOutOfBoundsError> {
        let msb_address = self.register_file.PC as usize;
        let lsb_address = (self.register_file.PC + 1) as usize;
        if lsb_address > RAM_SIZE {
            return Err(RAMOutOfBoundsError::new());
        }
        Ok((self.ram[msb_address], self.ram[lsb_address]))
    }

    pub fn execute_cycle(&mut self, keyboard: impl Keyboard, screen: &mut Screen) -> Result<(), Box<dyn Error>> {
        let (msb, lsb) = self.get_next_instruction_bytes()?;

        let instruction = decoder::decode_instruction(msb, lsb)?;

        executor::execute_instruction(
            instruction,
            &mut self.register_file,
            &mut self.ram,
            &keyboard,
            screen,
        )?;

        match instruction {
            Instruction::JP(_) | Instruction::JPV0(_) => {}
            _ => {
                self.register_file.PC += 2;
            }
        };

        Ok(())
    }

    pub fn inspect_register_file(&self) -> RegisterFile {
        self.register_file
    }

    pub fn inspect_next_instruction(&self) -> Result<Instruction, Box<dyn std::error::Error>> {
        let (msb, lsb) = self.get_next_instruction_bytes()?;
        Ok(decoder::decode_instruction(msb, lsb)?)
    }

    pub fn inspect_memory(&self) -> &RAM {
        &self.ram
    }

    pub fn tick_timers(&mut self) {
        if self.register_file.DT > 0 {
            self.register_file.DT -= 1;
        }

        if self.register_file.ST > 0 {
            self.register_file.ST -= 1;
        }
    }
}
