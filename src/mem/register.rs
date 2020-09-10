#![allow(non_snake_case)]

use serde::Serialize;

#[derive(Debug, Copy, Clone)]
pub enum Register {
    V(VRegister),
    Other(OtherRegister),
}

#[derive(Debug, Copy, Clone, Serialize)]
pub enum VRegister {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

#[derive(Debug, Copy, Clone)]
pub enum OtherRegister {
    DT,
    ST,
    I,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct RegisterFile {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    vA: u8,
    vB: u8,
    vC: u8,
    vD: u8,
    vE: u8,
    vF: u8,
    pub I: u16,
    pub PC: u16,
    pub SP: u8,
    pub DT: u8,
    pub ST: u8,
}

impl RegisterFile {
    pub fn get_v_register(&self, name: VRegister) -> u8 {
        match name {
            VRegister::V0 => self.v0,
            VRegister::V1 => self.v1,
            VRegister::V2 => self.v2,
            VRegister::V3 => self.v3,
            VRegister::V4 => self.v4,
            VRegister::V5 => self.v5,
            VRegister::V6 => self.v6,
            VRegister::V7 => self.v7,
            VRegister::V8 => self.v8,
            VRegister::V9 => self.v9,
            VRegister::VA => self.vA,
            VRegister::VB => self.vB,
            VRegister::VC => self.vC,
            VRegister::VD => self.vD,
            VRegister::VE => self.vE,
            VRegister::VF => self.vF,
        }
    }
    pub fn set_v_register(&mut self, name: VRegister, val: u8) {
        match name {
            VRegister::V0 => self.v0 = val,
            VRegister::V1 => self.v1 = val,
            VRegister::V2 => self.v2 = val,
            VRegister::V3 => self.v3 = val,
            VRegister::V4 => self.v4 = val,
            VRegister::V5 => self.v5 = val,
            VRegister::V6 => self.v6 = val,
            VRegister::V7 => self.v7 = val,
            VRegister::V8 => self.v8 = val,
            VRegister::V9 => self.v9 = val,
            VRegister::VA => self.vA = val,
            VRegister::VB => self.vB = val,
            VRegister::VC => self.vC = val,
            VRegister::VD => self.vD = val,
            VRegister::VE => self.vE = val,
            VRegister::VF => self.vF = val,
        }
    }

    pub fn new() -> RegisterFile {
        RegisterFile {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            vA: 0,
            vB: 0,
            vC: 0,
            vD: 0,
            vE: 0,
            vF: 0,
            I: 0,
            DT: 0,
            ST: 0,
            PC: 0,
            SP: 0,
        }
    }
}
