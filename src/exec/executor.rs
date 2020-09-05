use crate::io::keys::Keyboard;
use crate::logic::instruction::{ByteOrVReg, Instruction};
use crate::mem::register::{RegisterFile, VRegister};
use crate::mem::RAM;

use std::error::Error;

use rand::prelude::*;
use rand::thread_rng;

#[derive(Debug)]
pub struct InvalidStackPointerError {
    msg: &'static str,
}

impl std::fmt::Display for InvalidStackPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for InvalidStackPointerError {}

impl InvalidStackPointerError {
    pub fn overflow() -> InvalidStackPointerError {
        InvalidStackPointerError {
            msg: "Stack overflow, up to 16 layers of functions can used",
        }
    }
    pub fn negative() -> InvalidStackPointerError {
        InvalidStackPointerError {
            msg: "Attempted pop of empty stack",
        }
    }
}

pub const MAX_STACK_FRAMES: usize = 16;
pub const STACK_FRAME_SIZE: usize = 2;
pub const STACK_SIZE: usize = MAX_STACK_FRAMES * STACK_FRAME_SIZE;

fn push_pc(ram: &mut RAM, sp: &mut u8, pc: u16) -> Result<(), InvalidStackPointerError> {
    if *sp as usize > STACK_SIZE {
        return Err(InvalidStackPointerError::overflow());
    }
    let pc_msb = ((pc & 0xFF00) >> 8) as u8;
    let pc_lsb = (pc & 0x00FF) as u8;
    ram[*sp as usize] = pc_msb;
    ram[(*sp + 1) as usize] = pc_lsb;
    *sp += 2;
    Ok(())
}

fn pop_pc(ram: &mut RAM, sp: &mut u8) -> Result<u16, InvalidStackPointerError> {
    if *sp < 2 {
        return Err(InvalidStackPointerError::negative());
    }
    let pc_lsb = ram[*sp as usize];
    let pc_msb = ram[(*sp - 1) as usize];
    *sp -= 2;

    let pc = ((pc_msb as u16) << 8) + pc_lsb as u16;
    Ok(pc)
}

fn get_val_from_byte_or_v_register(
    byte_or_register: ByteOrVReg,
    register_file: &RegisterFile,
) -> u8 {
    match byte_or_register {
        ByteOrVReg::Byte(val) => val,
        ByteOrVReg::Register(reg) => register_file.get_v_register(reg),
    }
}

pub fn execute_instruction(
    instruction: Instruction,
    register_file: &mut RegisterFile,
    ram: &mut RAM,
    keyboard: &dyn Keyboard,
) -> Result<(), Box<dyn Error>> {
    match instruction {
        Instruction::LD(reg, byte_or_reg) => {
            let val = get_val_from_byte_or_v_register(byte_or_reg, register_file);
            register_file.set_v_register(reg, val);
        }
        Instruction::LDI(addr) => register_file.I = addr,
        Instruction::JP(addr) => register_file.PC = addr,
        Instruction::SE(reg, byte_or_reg) => {
            let reg_val = register_file.get_v_register(reg);
            let val = get_val_from_byte_or_v_register(byte_or_reg, register_file);

            if reg_val == val {
                register_file.PC += 2;
            }
        }
        Instruction::SNE(reg, byte_or_reg) => {
            let reg_val = register_file.get_v_register(reg);
            let val = get_val_from_byte_or_v_register(byte_or_reg, register_file);

            if reg_val != val {
                register_file.PC += 2;
            }
        }
        Instruction::ADD(reg, byte_or_reg) => {
            let reg_val = register_file.get_v_register(reg);
            let val = get_val_from_byte_or_v_register(byte_or_reg, register_file);

            let (result, carry) = reg_val.overflowing_add(val);

            register_file.set_v_register(VRegister::VF, carry as u8);
            register_file.set_v_register(reg, result);
        }
        Instruction::SUB(reg_a, reg_b) => {
            let reg_a_val = register_file.get_v_register(reg_a);
            let reg_b_val = register_file.get_v_register(reg_b);

            let (result, borrow) = reg_a_val.overflowing_sub(reg_b_val);

            register_file.set_v_register(VRegister::VF, (!borrow) as u8);
            register_file.set_v_register(reg_a, result);
        }
        Instruction::SUBN(reg_a, reg_b) => {
            let reg_a_val = register_file.get_v_register(reg_a);
            let reg_b_val = register_file.get_v_register(reg_b);

            let (result, borrow) = reg_b_val.overflowing_sub(reg_a_val);

            register_file.set_v_register(VRegister::VF, (!borrow) as u8);
            register_file.set_v_register(reg_a, result);
        }
        Instruction::OR(reg_a, reg_b) => {
            let reg_a_val = register_file.get_v_register(reg_a);
            let reg_b_val = register_file.get_v_register(reg_b);

            let result = reg_a_val | reg_b_val;
            register_file.set_v_register(reg_a, result);
        }
        Instruction::AND(reg_a, reg_b) => {
            let reg_a_val = register_file.get_v_register(reg_a);
            let reg_b_val = register_file.get_v_register(reg_b);

            let result = reg_a_val & reg_b_val;
            register_file.set_v_register(reg_a, result);
        }
        Instruction::XOR(reg_a, reg_b) => {
            let reg_a_val = register_file.get_v_register(reg_a);
            let reg_b_val = register_file.get_v_register(reg_b);

            let result = reg_a_val ^ reg_b_val;
            register_file.set_v_register(reg_a, result);
        }
        Instruction::SHL(reg) => {
            let reg_val = register_file.get_v_register(reg);

            let (result, flag) = reg_val.overflowing_shl(1);

            register_file.set_v_register(VRegister::VF, flag as u8);
            register_file.set_v_register(reg, result);
        }
        Instruction::SHR(reg) => {
            let reg_val = register_file.get_v_register(reg);

            let (result, flag) = reg_val.overflowing_shr(1);

            register_file.set_v_register(VRegister::VF, flag as u8);
            register_file.set_v_register(reg, result);
        }
        Instruction::JPV0(addr) => {
            let v0_val = register_file.get_v_register(VRegister::V0);

            let target = v0_val as u16 + addr;

            register_file.PC = target;
        }
        Instruction::RND(reg, byte) => {
            let mut rng = thread_rng();
            let random_byte = rng.next_u32() as u8;

            let result = random_byte & byte;
            register_file.set_v_register(reg, result);
        }
        Instruction::LD_FROM_DT(reg) => {
            let val = register_file.DT;

            register_file.set_v_register(reg, val);
        }
        Instruction::LD_TO_DT(reg) => {
            let val = register_file.get_v_register(reg);

            register_file.DT = val;
        }
        Instruction::LDST(reg) => {
            let val = register_file.get_v_register(reg);

            register_file.ST = val;
        }
        Instruction::ADDI(reg) => {
            let val = register_file.get_v_register(reg);

            let result = (val as u16).wrapping_add(register_file.I);

            register_file.I = result;
        }
        Instruction::CALL(addr) => {
            push_pc(ram, &mut register_file.SP, register_file.PC)?;
            register_file.PC = addr;
        }
        Instruction::RET => {
            let pc = pop_pc(ram, &mut register_file.SP)?;
            register_file.PC = pc;
        }
        Instruction::SKP(reg) => {
            let val = register_file.get_v_register(reg);
            if keyboard.is_key_pressed(val) {
                register_file.PC += 2;
            }
        },
        Instruction::SKNP(reg) => {
            let val = register_file.get_v_register(reg);
            if !keyboard.is_key_pressed(val) {
                register_file.PC += 2;
            }
        },
        _ => (),
    };
    Ok(())
}
