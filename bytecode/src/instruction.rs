// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::opcodes::Op;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{ErrorKind, Read, Write};

#[derive(Clone, Copy)]
pub struct Instruction {
    pub opcode: Op,

    pub a: u8,
    pub b: u8,
    pub c: u8,
}

impl Instruction {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[self.opcode as u8])?;
        writer.write_all(&[self.a, self.b, self.c])?;

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let mut opcode_bytes = [0u8; 1];
        let mut aux_bytes = [0u8; 3];

        reader.read_exact(&mut opcode_bytes)?;
        reader.read_exact(&mut aux_bytes)?;

        let opcode =
            Op::try_from(u8::from_le_bytes(opcode_bytes)).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        let a = aux_bytes[0];
        let b = aux_bytes[1];
        let c = aux_bytes[2];

        Ok(Self { opcode, a, b, c })
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op_name = match self.opcode {
            Op::LoadConst => "LOADCONST",
            Op::Move => "MOVE",
            Op::Return => "RETURN",
            Op::ReturnVoid => "RETURNVOID",
            Op::Add => "ADD",
            Op::Subtract => "SUBTRACT",
            Op::Multiply => "MULTIPLY",
            Op::Divide => "DIVIDE",
            Op::Exponentiate => "EXPONENTIATE",
            Op::Jump => "JUMP",
            Op::JumpIfTruthy => "JUMPIFTRUTHY",
            Op::JumpIfFalsy => "JUMPIFFALSY",
            Op::LogicalNot => "LOGICALNOT",
            Op::Negate => "NEGATE",
            Op::GreaterThan => "GREATERTHAN",
            Op::GreaterEqualThan => "GREATEREQUALTHAN",
            Op::LessThan => "LESSTHAN",
            Op::LessEqualThan => "LESSEQUALTHAN",
            Op::Equals => "EQUALS",
            Op::NotEquals => "NOTEQUALS",
            Op::Modulo => "MODULO",
            Op::Call => "CALL",
        };

        write!(f, "{} {} {} {}", op_name, self.a, self.b, self.c)
    }
}
