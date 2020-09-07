use crate::io::keys::{ Key, Keyboard };
use crate::io::screen::Screen;
use crate::logic::instruction::{ByteOrVReg, Instruction};
use crate::mem::register::{RegisterFile, VRegister};
use super::cpu::{ RAMOutOfBoundsError, RAM_DIGIT_SPRITE_START, DIGIT_SPRITE_SIZE };
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
    let pc_lsb = ram[(*sp - 1) as usize];
    let pc_msb = ram[(*sp - 2) as usize];
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

fn get_pressed_key(keyboard: &dyn Keyboard) -> Option<Key> {
    for key in 0x0..0xF {
        if keyboard.is_key_pressed(key) {
            return Some(key);
        }
    }

    None
}

fn get_v_register_range(end: VRegister) -> Vec<VRegister> {
    match end {
        VRegister::V0 => vec![VRegister::V0],
        VRegister::V1 => vec![VRegister::V0, VRegister::V1],
        VRegister::V2 => vec![VRegister::V0, VRegister::V1, VRegister::V2],
        VRegister::V3 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3],
        VRegister::V4 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4],
        VRegister::V5 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5],
        VRegister::V6 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6],
        VRegister::V7 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7],
        VRegister::V8 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8],
        VRegister::V9 => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9],
        VRegister::VA => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA],
        VRegister::VB => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA, VRegister::VB],
        VRegister::VC => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA, VRegister::VB, VRegister::VC],
        VRegister::VD => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA, VRegister::VB, VRegister::VC, VRegister::VD],
        VRegister::VE => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA, VRegister::VB, VRegister::VC, VRegister::VD, VRegister::VE],
        VRegister::VF => vec![VRegister::V0, VRegister::V1, VRegister::V2, VRegister::V3, VRegister::V4, VRegister::V5, VRegister::V6, VRegister::V7, VRegister::V8, VRegister::V9, VRegister::VA, VRegister::VB, VRegister::VC, VRegister::VD, VRegister::VE, VRegister::VF],
    }
}

pub fn execute_instruction(
    instruction: Instruction,
    register_file: &mut RegisterFile,
    ram: &mut RAM,
    keyboard: &dyn Keyboard,
    screen: &mut Screen
) -> Result<(), Box<dyn Error>> {
    match instruction {
        Instruction::CLS => {
            screen.clear();
        },
        Instruction::LD(reg, byte_or_reg) => {
            let val = get_val_from_byte_or_v_register(byte_or_reg, register_file);
            register_file.set_v_register(reg, val);
        },
        Instruction::LDI(addr) => register_file.I = addr & 0x0FFF,
        Instruction::LDK(reg) => {
            match get_pressed_key(&keyboard) {
                Some(key) => register_file.set_v_register(reg, key),
                None => register_file.PC -= 2,
            }
        },
        Instruction::LDARR(end_reg) => {
            for (i, reg) in get_v_register_range(end_reg).into_iter().enumerate() {
                let val = register_file.get_v_register(reg);
                let dest_addr = (register_file.I as usize) + i;
                ram[dest_addr] = val;
            }
        },
        Instruction::RDARR(end_reg) => {
            for (i, reg) in get_v_register_range(end_reg).into_iter().enumerate() {
                let src_addr = (register_file.I as usize) + i;
                register_file.set_v_register(reg, ram[src_addr]);
            }
        },
        Instruction::LDF(reg) => {
            let val = register_file.get_v_register(reg);

            register_file.I = (RAM_DIGIT_SPRITE_START + (DIGIT_SPRITE_SIZE * (val as usize))) as u16;
        },
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
        Instruction::LDBCD(reg) => {
            let val = register_file.get_v_register(reg);
            let hundreds = val / 100;
            let tens = (val - hundreds * 100) / 10;
            let ones = val - hundreds * 100 - tens * 10;

            let i_val = register_file.I;
            if (i_val + 2) as usize > ram.len() {
                return Err(Box::new(RAMOutOfBoundsError::new()));
            }

            ram[i_val as usize] = hundreds;
            ram[(i_val + 1) as usize] = tens;
            ram[(i_val + 2) as usize] = ones;
        },
        Instruction::DRW(reg_a, reg_b, n) => {
            let mut sprite = vec![];
            for i in 0..n {
                let addr = register_file.I + (i as u16);
                sprite.push(ram[addr as usize])
            }
            let x = register_file.get_v_register(reg_a);
            let y = register_file.get_v_register(reg_b);
            let vf_val = screen.draw(x, y, sprite)?;

            register_file.set_v_register(VRegister::VF, vf_val as u8);
        },
    };
    Ok(())
}
