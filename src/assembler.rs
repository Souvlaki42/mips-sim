use std::{collections::HashMap, ffi::CString, iter::Peekable, slice::Iter, str::FromStr};

use thiserror::Error;

use crate::{
    address::Address,
    cli::CLI,
    lexer::{Directive, Token, TokenizerError, tokenize},
    registers::{Register, RegisterError},
};

pub const BASE_TEXT_ADDR: Address = Address(0x0040_0000);
pub const BASE_DATA_ADDR: Address = Address(0x1001_0000);
pub const MEMORY_SIZE: usize = 64 * 1024;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Segment {
    Text,
    Data,
}

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Unknown directive")]
    UnknownDirective,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Entrypoint missing")]
    EntrypointMissing,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid register: {0}")]
    InvalidRegister(#[from] RegisterError),
    #[error("Invalid label: {0}")]
    InvalidLabel(String),
    #[error("Invalid string")]
    InvalidString,
    #[error("Invalid byte value")]
    InvalidByteValue,
    #[error("Tokenization failed: {0}")]
    TokenizationFailed(#[from] TokenizerError),
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    address: Address,
    segment: Segment,
}

pub struct Assembler {
    symbols: HashMap<String, Symbol>,
    data_addr: Address,
    text_addr: Address,
    entry_point: Option<String>,
    memory: Vec<u8>,
    text_lines: Vec<Instruction>,
    current_segment: Segment,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    AddImmediate {
        res: Register,
        reg: Register,
        imm: i32,
    },
    AddUnsigned {
        res: Register,
        reg: Register,
        ret: Register,
    },
    LoadUpperImmediate {
        res: Register,
        imm: i32,
    },
    OrImmediate {
        res: Register,
        reg: Register,
        imm: i32,
    },
    SystemCall,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            data_addr: BASE_DATA_ADDR,
            text_addr: BASE_TEXT_ADDR,
            entry_point: None,
            memory: vec![0; MEMORY_SIZE],
            text_lines: Vec::new(),
            current_segment: Segment::Text,
        }
    }

    // TODO: Add support for forward references
    pub fn assemble(&mut self, args: &CLI) -> Result<(), AssemblerError> {
        let tokenized = tokenize(&args.file)?;

        for line_tokens in tokenized {
            if args.tokens {
                println!("{:?}", line_tokens);
            }

            let mut tokens = line_tokens.iter().peekable();

            if let Some(Token::Label { name, decl: true }) = tokens.peek() {
                let addr = match self.current_segment {
                    Segment::Data => self.data_addr,
                    Segment::Text => self.text_addr,
                };
                self.symbols.insert(
                    name.clone(),
                    Symbol {
                        address: addr,
                        segment: self.current_segment,
                    },
                );
                tokens.next();
            }

            match tokens.next() {
                Some(Token::Directive { kind }) => self.handle_directive(kind, &mut tokens)?,
                Some(token) if matches!(token, Token::Operator { .. }) => {
                    let expanded = self.expand_instruction(line_tokens)?;
                    self.text_lines.extend(&expanded);
                    if args.instructions {
                        println!("{:?}", expanded);
                    }
                }
                None => continue,
                _ => return Err(AssemblerError::InvalidToken),
            }
        }

        Ok(())
    }

    pub fn expand_instruction(
        &mut self,
        tokens: Vec<Token>,
    ) -> Result<Vec<Instruction>, AssemblerError> {
        let mut iter = tokens.iter().peekable();
        if let Some(Token::Operator { value }) = iter.next() {
            let value_str = value.as_str();
            match value_str {
                "syscall" => return Ok(vec![Instruction::SystemCall]),
                "addi" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate(&mut iter)?;
                    return Ok(vec![Instruction::AddImmediate { res, reg, imm }]);
                }
                "addu" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    let ret = self.parse_register(&mut iter)?;
                    return Ok(vec![Instruction::AddUnsigned { res, reg, ret }]);
                }
                "lui" => {
                    let res = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate(&mut iter)?;
                    return Ok(vec![Instruction::LoadUpperImmediate { res, imm }]);
                }
                "ori" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate(&mut iter)?;
                    return Ok(vec![Instruction::OrImmediate { res, reg, imm }]);
                }
                "move" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    return Ok(vec![Instruction::AddUnsigned {
                        res,
                        reg,
                        ret: Register::ZERO,
                    }]);
                }
                "li" => {
                    let res = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate(&mut iter)?;

                    if imm >= -32768 && imm <= 32767 {
                        return Ok(vec![Instruction::AddImmediate {
                            res,
                            reg: Register::ZERO,
                            imm,
                        }]);
                    } else if (imm & 0xFFFF) == 0 {
                        return Ok(vec![Instruction::LoadUpperImmediate {
                            res,
                            imm: (imm >> 16),
                        }]);
                    } else {
                        let high = (imm >> 16) + if (imm & 0x8000) != 0 { 1 } else { 0 };
                        let low = imm & 0xFFFF;
                        return Ok(vec![
                            Instruction::LoadUpperImmediate { res, imm: high },
                            Instruction::AddImmediate {
                                res,
                                reg: res,
                                imm: low,
                            },
                        ]);
                    }
                }
                "la" => {
                    let res = self.parse_register(&mut iter)?;
                    let label = self.parse_label(&mut iter)?;
                    let symbol = self
                        .symbols
                        .get(&label)
                        .ok_or(AssemblerError::InvalidLabel(label.clone()))?;

                    if symbol.segment != Segment::Data {
                        return Err(AssemblerError::InvalidLabel(label.clone()));
                    }

                    let high = (*symbol).address >> 16;
                    let low = (*symbol).address & 0xffff.into();

                    return Ok(vec![
                        Instruction::LoadUpperImmediate {
                            res,
                            imm: high.into(),
                        },
                        Instruction::OrImmediate {
                            res,
                            reg: res,
                            imm: low.into(),
                        },
                    ]);
                }
                _ => {}
            }
        }
        Err(AssemblerError::InvalidInstruction)
    }

    pub fn get_entry_point(&self) -> Address {
        match &self.entry_point {
            Some(entry) => match self.symbols.get(entry) {
                Some(symbol) => symbol.address,
                None => BASE_TEXT_ADDR,
            },
            None => BASE_TEXT_ADDR,
        }
    }

    pub fn take_memory(&self) -> Vec<u8> {
        self.memory.clone()
    }

    pub fn get_instructions(&self) -> HashMap<Address, Instruction> {
        self.text_lines
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, inst)| {
                let addr = BASE_TEXT_ADDR + i * 4;
                (addr, inst)
            })
            .collect()
    }

    fn handle_directive(
        &mut self,
        kind: &Directive,
        tokens: &mut Peekable<Iter<Token>>,
    ) -> Result<(), AssemblerError> {
        match kind {
            Directive::DataDirective => {
                self.current_segment = Segment::Data;
                Ok(())
            }
            Directive::TextDirective => {
                self.current_segment = Segment::Text;
                Ok(())
            }
            Directive::GlobalDirective => {
                if let Some(Token::Label { name, decl: false }) = tokens.next() {
                    self.entry_point = Some(name.clone());
                    Ok(())
                } else {
                    Err(AssemblerError::EntrypointMissing)
                }
            }
            Directive::AsciizDirective => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(&value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes_with_nul();
                    let start_offset = self.data_addr - BASE_DATA_ADDR;
                    let end_offset = start_offset + bytes.len();
                    self.memory
                        .resize(std::cmp::max(self.memory.len(), end_offset), 0);
                    self.memory[start_offset..end_offset].copy_from_slice(&bytes);
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            Directive::AsciiDirective => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(&value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes();
                    let start_offset = self.data_addr - BASE_DATA_ADDR;
                    let end_offset = start_offset + bytes.len();
                    self.memory
                        .resize(std::cmp::max(self.memory.len(), end_offset), 0);
                    self.memory[start_offset..end_offset].copy_from_slice(&bytes);
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            Directive::ByteDirective => {
                while let Some(Token::Number { value }) = tokens.next() {
                    if *value < -128 || *value > 255 {
                        return Err(AssemblerError::InvalidByteValue);
                    }

                    let byte_val = *value as u8;
                    let offset: usize = (self.data_addr - BASE_DATA_ADDR).into();

                    if offset >= self.memory.len() {
                        self.memory.resize(offset + 1, 0);
                    }

                    self.memory[offset] = byte_val;
                    self.data_addr += 1;
                }
                Ok(())
            }
            _ => Err(AssemblerError::UnknownDirective),
        }
    }

    fn parse_register(&self, iter: &mut Peekable<Iter<Token>>) -> Result<Register, AssemblerError> {
        match iter.next() {
            Some(Token::Register { value }) => value
                .parse::<Register>()
                .map_err(|e| AssemblerError::InvalidRegister(e)),
            _ => Err(AssemblerError::InvalidInstruction),
        }
    }

    fn parse_immediate(&self, iter: &mut Peekable<Iter<Token>>) -> Result<i32, AssemblerError> {
        match iter.next() {
            Some(Token::Number { value }) => Ok(*value),
            _ => Err(AssemblerError::InvalidInstruction),
        }
    }

    fn parse_label(&self, iter: &mut Peekable<Iter<Token>>) -> Result<String, AssemblerError> {
        match iter.next() {
            Some(Token::Label { name, decl: false }) => Ok(name.clone()),
            _ => Err(AssemblerError::InvalidLabel("Not a label".to_string())),
        }
    }
}
