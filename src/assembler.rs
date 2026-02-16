use std::{collections::HashMap, ffi::CString, iter::Peekable, slice::Iter, str::FromStr};

use thiserror::Error;

use crate::{
    address::Address,
    args::Args,
    instructions::Instruction,
    lexer::{Directive, Token, TokenizerError, tokenize},
    registers::{Register, RegisterError},
};

pub const BASE_TEXT_ADDR: Address = Address(0x0040_0000);
pub const BASE_DATA_ADDR: Address = Address(0x1001_0000);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Segment {
    Text,
    Data,
}

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Unknown directive")]
    InvalidToken,
    #[error("Entrypoint missing")]
    EntrypointMissing,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid immediate: {0:?}")]
    InvalidImmediate(Option<i32>),
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

pub struct Assembler<'a> {
    memory: &'a mut HashMap<Address, u8>,
    symbols: HashMap<String, Symbol>,
    data_addr: Address,
    text_addr: Address,
    entry_point: Option<String>,
    current_segment: Segment,
}

impl<'a> Assembler<'a> {
    pub fn new(memory: &'a mut HashMap<Address, u8>) -> Self {
        Self {
            symbols: HashMap::new(),
            data_addr: BASE_DATA_ADDR,
            text_addr: BASE_TEXT_ADDR,
            entry_point: None,
            memory,
            current_segment: Segment::Text,
        }
    }

    // TODO: Add support for forward references
    pub fn assemble(&mut self, args: &Args) -> Result<(), AssemblerError> {
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
                Some(Token::Operator { .. }) => {
                    let expanded = self.expand_instruction(line_tokens)?;

                    for inst in &expanded {
                        let bytes = inst.encode().to_le_bytes();
                        for (i, &byte) in bytes.iter().enumerate() {
                            let addr = self.text_addr + i;
                            self.memory.insert(addr, byte);
                        }
                        self.text_addr += bytes.len();
                    }

                    if args.instructions {
                        println!("{:?}", expanded);
                    }
                }
                None => continue,
                _ => return Err(AssemblerError::InvalidToken),
            }
        }

        if args.memory {
            println!("{:?}", self.memory);
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
                    let imm = self.parse_immediate_i16(&mut iter)?;
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
                    let imm = self.parse_immediate_i16(&mut iter)?;
                    return Ok(vec![Instruction::LoadUpperImmediate { res, imm }]);
                }
                "ori" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate_u16(&mut iter)?;
                    return Ok(vec![Instruction::OrImmediate { res, reg, imm }]);
                }
                "move" => {
                    let res = self.parse_register(&mut iter)?;
                    let reg = self.parse_register(&mut iter)?;
                    return Ok(vec![Instruction::AddUnsigned {
                        res,
                        reg,
                        ret: Register::Zero,
                    }]);
                }
                "li" => {
                    let res = self.parse_register(&mut iter)?;
                    let imm = self.parse_immediate_i32(&mut iter)?;

                    if let Ok(imm) = i16::try_from(imm) {
                        return Ok(vec![Instruction::AddImmediate {
                            res,
                            reg: Register::Zero,
                            imm,
                        }]);
                    } else if (imm & 0xFFFF) == 0 {
                        return Ok(vec![Instruction::LoadUpperImmediate {
                            res,
                            imm: (imm >> 16) as i16,
                        }]);
                    } else {
                        let high = (imm >> 16) + if (imm & 0x8000) != 0 { 1 } else { 0 };
                        let low = imm & 0xFFFF;
                        return Ok(vec![
                            Instruction::LoadUpperImmediate {
                                res,
                                imm: high as i16,
                            },
                            Instruction::AddImmediate {
                                res,
                                reg: res,
                                imm: low as i16,
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

                    let high = symbol.address >> 16;
                    let low = symbol.address & 0xffff.into();

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
        self.entry_point
            .as_ref()
            .and_then(|e| self.symbols.get(e))
            .map(|s| s.address)
            .unwrap_or(BASE_TEXT_ADDR)
    }

    fn handle_directive(
        &mut self,
        kind: &Directive,
        tokens: &mut Peekable<Iter<Token>>,
    ) -> Result<(), AssemblerError> {
        match kind {
            Directive::Data => {
                self.current_segment = Segment::Data;
                Ok(())
            }
            Directive::Text => {
                self.current_segment = Segment::Text;
                Ok(())
            }
            Directive::Global => {
                if let Some(Token::Label { name, decl: false }) = tokens.next() {
                    self.entry_point = Some(name.clone());
                    Ok(())
                } else {
                    Err(AssemblerError::EntrypointMissing)
                }
            }
            Directive::Asciiz => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes_with_nul();
                    for (i, &byte) in bytes.iter().enumerate() {
                        self.memory.insert(self.data_addr + i, byte);
                    }
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            Directive::Ascii => {
                if let Some(Token::Text { value }) = tokens.next() {
                    let bytes = CString::from_str(value)
                        .map_err(|_| AssemblerError::InvalidString)?
                        .into_bytes();
                    for (i, &byte) in bytes.iter().enumerate() {
                        self.memory.insert(self.data_addr + i, byte);
                    }
                    self.data_addr += bytes.len();
                    Ok(())
                } else {
                    Err(AssemblerError::InvalidToken)
                }
            }
            Directive::Byte => {
                while let Some(Token::Number { value }) = tokens.next() {
                    if ((i8::MIN as i32)..=(u8::MAX as i32)).contains(value) {
                        return Err(AssemblerError::InvalidByteValue);
                    }

                    self.memory.insert(self.data_addr, *value as u8);
                    self.data_addr += 1;
                }
                Ok(())
            }
            Directive::Word => {
                let offset = self.data_addr % 4;
                if offset != 0 {
                    self.data_addr += (4 - offset) as usize;
                }

                while let Some(Token::Number { value }) = tokens.next() {
                    let bytes = value.to_le_bytes();

                    for (i, &byte) in bytes.iter().enumerate() {
                        self.memory.insert(self.data_addr + i, byte);
                    }
                    self.data_addr += bytes.len();
                }
                Ok(())
            }
        }
    }

    fn parse_register(&self, iter: &mut Peekable<Iter<Token>>) -> Result<Register, AssemblerError> {
        match iter.next() {
            Some(Token::Register { value }) => value
                .parse::<Register>()
                .map_err(AssemblerError::InvalidRegister),
            _ => Err(AssemblerError::InvalidInstruction),
        }
    }

    fn parse_immediate_i32(&self, iter: &mut Peekable<Iter<Token>>) -> Result<i32, AssemblerError> {
        match iter.next() {
            Some(Token::Number { value }) => Ok(*value),
            _ => Err(AssemblerError::InvalidImmediate(None)),
        }
    }

    fn parse_immediate_i16(&self, iter: &mut Peekable<Iter<Token>>) -> Result<i16, AssemblerError> {
        match iter.next() {
            Some(Token::Number { value }) => match i16::try_from(*value) {
                Ok(value) => Ok(value),
                Err(_) => Err(AssemblerError::InvalidImmediate(Some(*value))),
            },
            _ => Err(AssemblerError::InvalidImmediate(None)),
        }
    }

    fn parse_immediate_u16(&self, iter: &mut Peekable<Iter<Token>>) -> Result<u16, AssemblerError> {
        match iter.next() {
            Some(Token::Number { value }) => match u16::try_from(*value) {
                Ok(value) => Ok(value),
                Err(_) => Err(AssemblerError::InvalidImmediate(Some(*value))),
            },
            _ => Err(AssemblerError::InvalidImmediate(None)),
        }
    }

    fn parse_label(&self, iter: &mut Peekable<Iter<Token>>) -> Result<String, AssemblerError> {
        match iter.next() {
            Some(Token::Label { name, decl: false }) => Ok(name.clone()),
            _ => Err(AssemblerError::InvalidLabel("Not a label".to_string())),
        }
    }
}
