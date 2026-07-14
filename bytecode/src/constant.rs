// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::chunk::Chunk;
use syntax::literal_value::LiteralValue;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{ErrorKind, Read, Write};

#[derive(Clone)]
pub enum Constant {
    Chunk(Chunk),
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

pub const STRING_TAG: u8 = 0x00;
pub const NUMBER_TAG: u8 = 0x01;
pub const BOOL_TAG: u8 = 0x02;
pub const NIL_TAG: u8 = 0x03;
pub const CHUNK_TAG: u8 = 0x04;

impl Constant {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            Constant::Chunk(c) => {
                writer.write_all(&[CHUNK_TAG])?;
                c.write(writer)?
            }
            Constant::String(s) => {
                writer.write_all(&[STRING_TAG])?; // 1 byte tag
                writer.write_all(&(s.len() as u32).to_le_bytes())?; // 4 byte length
                writer.write_all(s.as_bytes())?
            }
            Constant::Number(n) => {
                writer.write_all(&[NUMBER_TAG])?; // 1 byte tag
                writer.write_all(&n.to_le_bytes())?; // 8 byte length (all numbers are f64)
            }
            Constant::Bool(n) => {
                writer.write_all(&[BOOL_TAG])?; // 1 byte tag
                writer.write_all(&[*n as u8])?; // 1 byte length (inefficient but whatever)
            }
            Constant::Nil => {
                writer.write_all(&[NIL_TAG])?; // 1 byte tag
            }
        }

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let mut tag_bytes = [0u8; 1];
        reader.read_exact(&mut tag_bytes)?;
        let tag = u8::from_le_bytes(tag_bytes);

        match tag {
            STRING_TAG => {
                let mut length_bytes = [0u8; 4];
                reader.read_exact(&mut length_bytes)?;
                let length = u32::from_le_bytes(length_bytes);

                let mut string_bytes = vec![0u8; length as usize];
                reader.read_exact(&mut string_bytes)?;

                let string = String::from_utf8(string_bytes)
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))?;

                Ok(Self::String(string))
            }
            NUMBER_TAG => {
                let mut number_bytes = [0u8; 8];
                reader.read_exact(&mut number_bytes)?;

                let number = f64::from_le_bytes(number_bytes);
                Ok(Self::Number(number))
            }
            BOOL_TAG => {
                let mut bool_bytes = [0u8; 1];
                reader.read_exact(&mut bool_bytes)?;

                match bool_bytes[0] {
                    0 => Ok(Constant::Bool(false)),
                    1 => Ok(Constant::Bool(true)),

                    _ => Err(io::Error::new(ErrorKind::InvalidData, "Invalid boolean value")),
                }
            }

            CHUNK_TAG => Ok(Constant::Chunk(Chunk::read(reader)?)),
            NIL_TAG => Ok(Constant::Nil),
            _ => Err(io::Error::new(ErrorKind::InvalidData, "Unknown constant tag")),
        }
    }
}

impl From<LiteralValue> for Constant {
    fn from(value: LiteralValue) -> Self {
        match value {
            LiteralValue::String(s) => Constant::String(s),
            LiteralValue::Number(n) => Constant::Number(n),
            LiteralValue::Bool(b) => Constant::Bool(b),
            LiteralValue::Nil => Constant::Nil,
        }
    }
}

impl Debug for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Chunk(_) => write!(f, "Constant(chunk))"),
            Constant::String(s) => write!(f, "Constant(\"{}\")", s),
            Constant::Number(n) => write!(f, "Constant({})", n),
            Constant::Bool(b) => write!(f, "Constant({})", b),
            Constant::Nil => write!(f, "Constant(nil)"),
        }
    }
}
