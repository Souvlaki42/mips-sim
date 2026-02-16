use thiserror::Error;

use crate::registers::{Register, RegisterError};

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    AddImmediate {
        res: Register,
        reg: Register,
        imm: i16,
    },
    AddUnsigned {
        res: Register,
        reg: Register,
        ret: Register,
    },
    LoadUpperImmediate {
        res: Register,
        imm: i16,
    },
    OrImmediate {
        res: Register,
        reg: Register,
        imm: u16,
    },
    SystemCall,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Instruction is not valid")]
    InvalidInstruction,
    #[error("Register is not valid")]
    InvalidRegister(#[from] RegisterError),
}

impl Instruction {
    pub fn encode(&self) -> u32 {
        match self {
            // I-type: opcode=8, rs=reg, rt=res, imm
            Instruction::AddImmediate { res, reg, imm } => {
                (8 << 26) | ((*reg as u32) << 21) | ((*res as u32) << 16) | (*imm as u16 as u32) // Preserve bit pattern
            }

            // R-type: op=0, rs=reg, rt=ret, rd=res, shamt=0, funct=0x21
            Instruction::AddUnsigned { res, reg, ret } => {
                0 | ((*reg as u32) << 21) | ((*ret as u32) << 16) | ((*res as u32) << 11) | 0x21
            }

            // I-type: opcode=15, rs=0, rt=res, imm
            Instruction::LoadUpperImmediate { res, imm } => {
                (15 << 26) | ((*res as u32) << 16) | (*imm as u16 as u32)
            }

            // I-type: opcode=13, rs=reg, rt=res, imm
            Instruction::OrImmediate { res, reg, imm } => {
                (13 << 26) | ((*reg as u32) << 21) | ((*res as u32) << 16) | (*imm as u32) // Already unsigned
            }

            // R-type: all zeros except funct=0x0c
            Instruction::SystemCall => 0x0000000c,
        }
    }

    pub fn decode(word: u32) -> Result<Self, DecodeError> {
        let opcode = (word >> 26) as u8;
        let rs = ((word >> 21) & 0x1F) as u8;
        let rt = ((word >> 16) & 0x1F) as u8;
        let rd = ((word >> 11) & 0x1F) as u8;
        let imm = (word & 0xFFFF) as u16;
        let funct = (word & 0x3F) as u8;

        match opcode {
            0 => {
                // R-type: check funct
                match funct {
                    0x0c => Ok(Instruction::SystemCall),
                    0x21 => Ok(Instruction::AddUnsigned {
                        res: Register::from_bits(rd)?,
                        reg: Register::from_bits(rs)?,
                        ret: Register::from_bits(rt)?,
                    }),
                    _ => Err(DecodeError::InvalidInstruction),
                }
            }
            8 => Ok(Instruction::AddImmediate {
                res: Register::from_bits(rt)?, // Note: rt is dest in I-type
                reg: Register::from_bits(rs)?,
                imm: imm as i16, // Cast to signed
            }),
            13 => Ok(Instruction::OrImmediate {
                res: Register::from_bits(rt)?,
                reg: Register::from_bits(rs)?,
                imm, // Already u16
            }),
            15 => Ok(Instruction::LoadUpperImmediate {
                res: Register::from_bits(rt)?,
                imm: imm as i16,
            }),
            _ => Err(DecodeError::InvalidInstruction),
        }
    }
}
