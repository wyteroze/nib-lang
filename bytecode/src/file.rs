// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::chunk::Chunk;
use crate::header::BytecodeHeader;
use std::io;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct BytecodeFile {
    pub header: BytecodeHeader,
    pub top_level: Chunk,
}

impl BytecodeFile {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        self.header.write(writer)?;
        self.top_level.write(writer)?;

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let header = BytecodeHeader::read(reader)?;
        let top_level = Chunk::read(reader)?;

        Ok(Self { header, top_level })
    }
}
