use super::instruction::{ByteOrVReg, Instruction};
use crate::mem::register::VRegister;

use std::error::Error;

#[derive(Debug)]
pub struct MalformedInstructionError {
    msg: String,
}

impl std::fmt::Display for MalformedInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for MalformedInstructionError {}

impl MalformedInstructionError {
    pub fn new(msb: u8, lsb: u8) -> MalformedInstructionError {
        MalformedInstructionError {
            msg: format!(
                "Malformed instruction received, (msb, lsb): ({:X}, {:X})",
                msb, lsb
            ),
        }
    }
}

fn create_addr_from_nybbles(f: u8, s: u8, t: u8) -> u16 {
    ((f as u16) << (2 * 4)) + ((s as u16) << 4) + t as u16
}

fn nybble_to_vregister(x: u8) -> VRegister {
    match x {
        0x0 => VRegister::V0,
        0x1 => VRegister::V1,
        0x2 => VRegister::V2,
        0x3 => VRegister::V3,
        0x4 => VRegister::V4,
        0x5 => VRegister::V5,
        0x6 => VRegister::V6,
        0x7 => VRegister::V7,
        0x8 => VRegister::V8,
        0x9 => VRegister::V9,
        0xA => VRegister::VA,
        0xB => VRegister::VB,
        0xC => VRegister::VC,
        0xD => VRegister::VD,
        0xE => VRegister::VE,
        0xF => VRegister::VF,
        _ => panic!("Invalid register nybble {}", x),
    }
}

pub fn decode_instruction(msb: u8, lsb: u8) -> Result<Instruction, MalformedInstructionError> {
    let first_nybble = (msb & 0xF0) >> 4;
    let second_nybble = msb & 0x0F;
    let third_nybble = (lsb & 0xF0) >> 4;
    let fourth_nybble = lsb & 0x0F;

    let instruction = ((msb as u16) << 8) + lsb as u16;

    let instruction = match first_nybble {
        0x0 => match instruction {
            0x00E0 => Instruction::CLS,
            0x00EE => Instruction::RET,
            _ => {
                return Err(MalformedInstructionError::new(msb, lsb));
            }
        },
        0x1 => Instruction::JP(create_addr_from_nybbles(
            second_nybble,
            third_nybble,
            fourth_nybble,
        )),
        0x2 => Instruction::CALL(create_addr_from_nybbles(
            second_nybble,
            third_nybble,
            fourth_nybble,
        )),
        0x3 => Instruction::SE(nybble_to_vregister(second_nybble), ByteOrVReg::Byte(lsb)),
        0x4 => Instruction::SNE(nybble_to_vregister(second_nybble), ByteOrVReg::Byte(lsb)),
        0x5 => Instruction::SE(
            nybble_to_vregister(second_nybble),
            ByteOrVReg::Register(nybble_to_vregister(third_nybble)),
        ),
        0x6 => Instruction::LD(nybble_to_vregister(second_nybble), ByteOrVReg::Byte(lsb)),
        0x7 => Instruction::ADD(nybble_to_vregister(second_nybble), ByteOrVReg::Byte(lsb)),
        0x8 => match fourth_nybble {
            0x0 => Instruction::LD(
                nybble_to_vregister(second_nybble),
                ByteOrVReg::Register(nybble_to_vregister(third_nybble)),
            ),
            0x1 => Instruction::OR(
                nybble_to_vregister(second_nybble),
                nybble_to_vregister(third_nybble),
            ),
            0x2 => Instruction::AND(
                nybble_to_vregister(second_nybble),
                nybble_to_vregister(third_nybble),
            ),
            0x3 => Instruction::XOR(
                nybble_to_vregister(second_nybble),
                nybble_to_vregister(third_nybble),
            ),
            0x4 => Instruction::ADD(
                nybble_to_vregister(second_nybble),
                ByteOrVReg::Register(nybble_to_vregister(third_nybble)),
            ),
            0x5 => Instruction::SUB(
                nybble_to_vregister(second_nybble),
                nybble_to_vregister(third_nybble),
            ),
            0x6 => Instruction::SHR(nybble_to_vregister(second_nybble)),
            0x7 => Instruction::SUBN(
                nybble_to_vregister(second_nybble),
                nybble_to_vregister(third_nybble),
            ),
            0xE => Instruction::SHL(nybble_to_vregister(second_nybble)),
            _ => {
                return Err(MalformedInstructionError::new(msb, lsb));
            }
        },
        0x9 => Instruction::SNE(
            nybble_to_vregister(second_nybble),
            ByteOrVReg::Register(nybble_to_vregister(third_nybble)),
        ),
        0xA => Instruction::LDI(create_addr_from_nybbles(
            second_nybble,
            third_nybble,
            fourth_nybble,
        )),
        0xB => Instruction::JPV0(create_addr_from_nybbles(
            second_nybble,
            third_nybble,
            fourth_nybble
        )),
        0xC => Instruction::RND(nybble_to_vregister(second_nybble), lsb),
        0xD => Instruction::DRW(
            nybble_to_vregister(second_nybble),
            nybble_to_vregister(third_nybble),
            fourth_nybble,
        ),
        0xE => match lsb {
            0x9E => Instruction::SKP(nybble_to_vregister(second_nybble)),
            0xA1 => Instruction::SKNP(nybble_to_vregister(second_nybble)),
            _ => {
                return Err(MalformedInstructionError::new(msb, lsb));
            }
        },
        0xF => match lsb {
            0x07 => Instruction::LD_FROM_DT(nybble_to_vregister(second_nybble)),
            0x0A => Instruction::LDK(nybble_to_vregister(second_nybble)),
            0x15 => Instruction::LD_TO_DT(nybble_to_vregister(second_nybble)),
            0x18 => Instruction::LDST(nybble_to_vregister(second_nybble)),
            0x1E => Instruction::ADDI(nybble_to_vregister(second_nybble)),
            0x29 => Instruction::LDF(nybble_to_vregister(second_nybble)),
            0x33 => Instruction::LDBCD(nybble_to_vregister(second_nybble)),
            0x55 => Instruction::LDARR(nybble_to_vregister(second_nybble)),
            0x65 => Instruction::RDARR(nybble_to_vregister(second_nybble)),
            _ => {
                return Err(MalformedInstructionError::new(msb, lsb));
            }
        },
        _ => {
            return Err(MalformedInstructionError::new(msb, lsb));
        }
    };

    Ok(instruction)
}
