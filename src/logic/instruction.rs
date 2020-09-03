#![allow(non_camel_case_types)]

use crate::mem::register::{ VRegister };

pub type Addr = u16;
pub type Nibble = u8;
pub type Byte = u8;

#[derive(Copy, Clone, Debug)]
pub enum ByteOrVReg {
    Byte(Byte),
    Register(VRegister),
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    CLS,
    RET,
    JP(Addr),
    CALL(Addr),
    SE(VRegister, ByteOrVReg),
    SNE(VRegister, ByteOrVReg),
    LD(VRegister, ByteOrVReg),
    ADD(VRegister, ByteOrVReg),
    ADDI(VRegister),
    OR(VRegister, VRegister),
    AND(VRegister, VRegister),
    XOR(VRegister, VRegister),
    SUB(VRegister, VRegister),
    SHR(VRegister),
    SUBN(VRegister, VRegister),
    SHL(VRegister),
    LDI(Addr),
    JPV0(Addr),
    RND(VRegister, Byte),
    DRW(VRegister, VRegister, Nibble),
    SKP(VRegister),
    SKNP(VRegister),
    LDK(VRegister),
    LDF(VRegister),
    LD_TO_DT(VRegister),
    LD_FROM_DT(VRegister),
    LDST(VRegister),
    LDBCD(VRegister),
    LDARR(VRegister),
    RDARR(VRegister),
}
