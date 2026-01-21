use std::{
    collections::HashMap,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    address::Address,
    assembler::{BASE_DATA_ADDR, Instruction},
    registers::{Register, RegisterError, RegisterFile},
};

#[derive(Debug, Error)]
pub enum SimulatorError {
    #[error("Register error: {0}")]
    RegisterError(#[from] RegisterError),
    #[error("Unknown syscall: {0}")]
    UnknownSyscall(u32),
    #[error("Exit with code {0}")]
    Exit(u32),
    #[error("No more instructions")]
    NoMoreInstructions,
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Wrong input type: {0}")]
    WrongInputType(String),
    #[error("Invalid system time: {0}")]
    InvalidSystemTime(#[from] SystemTimeError),
}

#[derive(Debug)]
pub struct Simulator<'a> {
    memory: &'a mut HashMap<Address, u8>,
    registers: RegisterFile,
    instructions: HashMap<Address, Instruction>,
    pc: Address,
}

impl<'a> Simulator<'a> {
    pub fn new(
        instructions: HashMap<Address, Instruction>,
        memory: &'a mut HashMap<Address, u8>,
        entry: Address,
    ) -> Self {
        Self {
            memory,
            registers: RegisterFile::default(),
            instructions,
            pc: entry,
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), SimulatorError> {
        match instruction {
            Instruction::AddImmediate { res, reg, imm } => {
                let value = self.registers.get(reg).wrapping_add(imm as u32);
                self.registers.set(res, value);
            }
            Instruction::LoadUpperImmediate { res, imm } => {
                let value = (imm as u32) << 16;
                self.registers.set(res, value);
            }
            Instruction::OrImmediate { res, reg, imm } => {
                let value = self.registers.get(reg) | (imm as u32);
                self.registers.set(res, value);
            }
            Instruction::SystemCall => {
                self.handle_syscall()?;
            }
            Instruction::AddUnsigned { res, reg, ret } => {
                let value = self
                    .registers
                    .get(reg)
                    .wrapping_add(self.registers.get(ret));
                self.registers.set(res, value);
            }
        }
        Ok(())
    }

    fn get_user_input(&mut self) -> Result<String, SimulatorError> {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| SimulatorError::IoError(e))?;
        input = input.trim().to_string();
        Ok(input)
    }

    fn handle_syscall(&mut self) -> Result<(), SimulatorError> {
        let v0 = self.registers.get(Register::V0);
        match v0 {
            1 => {
                let value = self.registers.get(Register::A0);
                print!("{}", value);
            }
            4 => {
                let addr: Address = self.registers.get(Register::A0).into();
                let offset: Address = addr - BASE_DATA_ADDR;

                let mut bytes = Vec::new();
                let mut i = offset;
                loop {
                    match self.memory.get(&i) {
                        Some(&byte) if byte != 0 => {
                            bytes.push(byte);
                            i += 1;
                        }
                        _ => break,
                    }
                }

                let s = String::from_utf8_lossy(&bytes);
                print!("{}", s);
            }
            5 => {
                let input = self.get_user_input()?;
                let value = input
                    .parse::<u32>()
                    .map_err(|_| SimulatorError::WrongInputType(input))?;
                self.registers.set(Register::V0, value);
            }
            10 => {
                return Err(SimulatorError::Exit(0));
            }
            17 => {
                let value = self.registers.get(Register::A0);
                return Err(SimulatorError::Exit(value));
            }
            30 => {
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| SimulatorError::InvalidSystemTime(e))?;

                let millis = duration.as_millis() as u64;

                let low = (millis & 0xFFFFFFFF) as u32;
                let high = (millis >> 32) as u32;

                self.registers.set(Register::A0, low);
                self.registers.set(Register::A1, high);
            }
            _ => {
                return Err(SimulatorError::UnknownSyscall(v0));
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), SimulatorError> {
        let instruction = self
            .instructions
            .get(&self.pc)
            .ok_or(SimulatorError::NoMoreInstructions)?
            .clone();

        self.execute_instruction(instruction)?;
        self.pc += 4;
        Ok(())
    }
}
