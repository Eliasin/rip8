use super::executor;
use crate::io::keys::Keyboard;
use crate::io::screen::Screen;
use crate::logic::decoder;
use crate::logic::instruction::Instruction;
use crate::mem::register::RegisterFile;
use crate::mem::{RAM, RAM_SIZE};

use std::error::Error;

const RAM_PROG_START: usize = 0x200;

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

    pub fn map_program(&mut self, program: Vec<u8>) -> Result<(), RAMOutOfBoundsError> {
        let program_length = program.len();
        if program_length > self.ram.len() - RAM_PROG_START {
            return Err(RAMOutOfBoundsError::new());
        }

        let program_area = &mut self.ram[RAM_PROG_START..RAM_PROG_START + program.len()];
        program_area.copy_from_slice(program.as_slice());

        Ok(())
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

    pub fn inspect_register_file(&self) -> String {
        format!("{:?}", self.register_file)
    }

    pub fn inspect_next_instruction(&self) -> Result<Instruction, Box<dyn std::error::Error>> {
        let (msb, lsb) = self.get_next_instruction_bytes()?;
        Ok(decoder::decode_instruction(msb, lsb)?)
    }

    pub fn inspect_memory(&self) -> String {
        format!("{:?}", self.ram)
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
