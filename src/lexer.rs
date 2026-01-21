use std::{fs::File, io::Read};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directive {
    DataDirective,
    TextDirective,
    GlobalDirective,
    AsciiDirective,
    AsciizDirective,
    ByteDirective,
    WordDirective,
}

#[derive(Debug, Clone)]
pub enum Token {
    Directive { kind: Directive },
    Register { value: String },
    Label { name: String, decl: bool },
    Number { value: i32 },
    Operator { value: String },
    Text { value: String },
}

#[derive(Debug, Error)]
pub enum TokenizerError {
    #[error("Failed to open file '{0}'")]
    OpenFileError(String),
    #[error("Failed to read file '{0}'")]
    ReadFileError(String),
    #[error("Unknown directive '{0}'")]
    UnknownDirective(String),
    #[error("Invalid byte ''{0}'")]
    InvalidByte(String),
}

fn parse_directive(token: &str) -> Result<Directive, TokenizerError> {
    match token {
        ".data" => Ok(Directive::DataDirective),
        ".text" => Ok(Directive::TextDirective),
        ".globl" => Ok(Directive::GlobalDirective),
        ".ascii" => Ok(Directive::AsciiDirective),
        ".asciiz" => Ok(Directive::AsciizDirective),
        ".byte" => Ok(Directive::ByteDirective),
        ".word" => Ok(Directive::WordDirective),
        other => Err(TokenizerError::UnknownDirective(other.to_string())),
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn tokenize(file_name: &str) -> Result<Vec<Vec<Token>>, TokenizerError> {
    let mut file =
        File::open(file_name).map_err(|_| TokenizerError::OpenFileError(file_name.to_string()))?;
    let mut contents = String::new();
    let mut all_tokens = Vec::new();

    file.read_to_string(&mut contents)
        .map_err(|_| TokenizerError::ReadFileError(file_name.to_string()))?;
    let lines = contents.lines();

    for mut line in lines {
        if line.starts_with("#") {
            continue;
        }

        if let Some((before, _)) = line.split_once("#") {
            line = before;
        }

        let mut tokens = Vec::new();
        let mut inside_string = false;
        let mut inside_byte = false;

        let raw_tokens: Vec<&str> = line
            .split(|c: char| {
                if c == '"' && !inside_byte {
                    inside_string = !inside_string;
                    false
                } else if c == '\'' && !inside_string {
                    inside_byte = !inside_byte;
                    false
                } else {
                    !inside_byte || !inside_string || c.is_whitespace() || c == ','
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        for (i, token) in raw_tokens.iter().enumerate() {
            if token.starts_with(".") {
                let directive = parse_directive(token)?;
                tokens.push(Token::Directive { kind: directive });
            } else if token.starts_with('"') && token.ends_with('"') {
                let value = unescape_string(&token[1..token.len() - 1]);
                tokens.push(Token::Text { value });
            } else if token.starts_with('\'') && token.ends_with('\'') {
                let unescaped = unescape_string(&token[1..token.len() - 1]);
                let bytes = unescaped.as_bytes();

                if bytes.len() != 1 {
                    return Err(TokenizerError::InvalidByte(unescaped));
                }

                tokens.push(Token::Number {
                    value: bytes[0] as i32,
                });
            } else if token.starts_with("0b")
                && let Ok(value) = i32::from_str_radix(&token[2..], 2)
            {
                tokens.push(Token::Number { value });
            } else if token.starts_with("0x")
                && let Ok(value) = i32::from_str_radix(&token[2..], 16)
            {
                tokens.push(Token::Number { value });
            } else if let Ok(value) = token.parse::<i32>() {
                tokens.push(Token::Number { value });
            } else if token.starts_with("$") {
                tokens.push(Token::Register {
                    value: token.to_string(),
                });
            } else if token.ends_with(":") {
                let name = token.trim_end_matches(":");
                tokens.push(Token::Label {
                    name: name.to_string(),
                    decl: true,
                });
            } else if i == 0 {
                tokens.push(Token::Operator {
                    value: token.to_string(),
                });
            } else {
                tokens.push(Token::Label {
                    name: token.to_string(),
                    decl: false,
                });
            }
        }
        all_tokens.push(tokens);
    }
    Ok(all_tokens)
}
