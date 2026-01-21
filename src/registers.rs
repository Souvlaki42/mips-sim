use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("There is no register named '{0}' in this processor")]
    NoSuchRegister(String),
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    Zero = 0,
    At = 1,
    V0 = 2,
    V1 = 3,
    A0 = 4,
    A1 = 5,
    A2 = 6,
    A3 = 7,
    T0 = 8,
    T1 = 9,
    T2 = 10,
    T3 = 11,
    T4 = 12,
    T5 = 13,
    T6 = 14,
    T7 = 15,
    S0 = 16,
    S1 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    T8 = 24,
    T9 = 25,
    K0 = 26,
    K1 = 27,
    Gp = 28,
    Sp = 29,
    Fp = 30,
    Ra = 31,
}

impl std::str::FromStr for Register {
    type Err = RegisterError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "$zero" | "$0" => Ok(Register::Zero),
            "$at" => Ok(Register::At),
            "$v0" => Ok(Register::V0),
            "$v1" => Ok(Register::V1),
            "$a0" => Ok(Register::A0),
            "$a1" => Ok(Register::A1),
            "$a2" => Ok(Register::A2),
            "$a3" => Ok(Register::A3),
            "$t0" => Ok(Register::T0),
            "$t1" => Ok(Register::T1),
            "$t2" => Ok(Register::T2),
            "$t3" => Ok(Register::T3),
            "$t4" => Ok(Register::T4),
            "$t5" => Ok(Register::T5),
            "$t6" => Ok(Register::T6),
            "$t7" => Ok(Register::T7),
            "$s0" => Ok(Register::S0),
            "$s1" => Ok(Register::S1),
            "$s2" => Ok(Register::S2),
            "$s3" => Ok(Register::S3),
            "$s4" => Ok(Register::S4),
            "$s5" => Ok(Register::S5),
            "$s6" => Ok(Register::S6),
            "$s7" => Ok(Register::S7),
            "$t8" => Ok(Register::T8),
            "$t9" => Ok(Register::T9),
            "$k0" => Ok(Register::K0),
            "$k1" => Ok(Register::K1),
            "$gp" => Ok(Register::Gp),
            "$sp" => Ok(Register::Sp),
            "$fp" => Ok(Register::Fp),
            "$ra" => Ok(Register::Ra),
            other => Err(RegisterError::NoSuchRegister(other.to_string())),
        }
    }
}

#[derive(Debug, Default)]
pub struct RegisterFile([u32; 32]);

impl RegisterFile {
    pub fn get(&self, r: Register) -> u32 {
        self.0[r as usize]
    }

    pub fn set(&mut self, r: Register, val: u32) {
        let idx = r as usize;
        if idx != 0 {
            self.0[idx] = val;
        }
    }
}
