// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::constant::Constant;
use crate::instruction::Instruction;
use std::io;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct ChunkInfo {
    pub arity: u8,
    pub instruction_count: u16,
    pub constant_count: u16,
}

impl ChunkInfo {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&[self.arity])?;
        writer.write_all(&self.instruction_count.to_le_bytes())?;
        writer.write_all(&self.constant_count.to_le_bytes())?;

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let mut arity_bytes = [0u8; 1];
        reader.read_exact(&mut arity_bytes)?;
        let arity = u8::from_le_bytes(arity_bytes);

        let mut instruction_count_bytes = [0u8; 2];
        reader.read_exact(&mut instruction_count_bytes)?;
        let instruction_count = u16::from_le_bytes(instruction_count_bytes);

        let mut constant_count_bytes = [0u8; 2];
        reader.read_exact(&mut constant_count_bytes)?;
        let constant_count = u16::from_le_bytes(constant_count_bytes);

        Ok(Self { arity, instruction_count, constant_count })
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub info: ChunkInfo,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        self.info.write(writer)?;

        for instruction in &self.instructions {
            instruction.write(writer)?;
        }

        for constant in &self.constants {
            constant.write(writer)?;
        }

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let info = ChunkInfo::read(reader)?;
        let mut instructions = Vec::new();
        let mut constants = Vec::new();

        for _ in 0..info.instruction_count {
            instructions.push(Instruction::read(reader)?);
        }

        for _ in 0..info.constant_count {
            constants.push(Constant::read(reader)?);
        }

        Ok(Self { info, instructions, constants })
    }
}
