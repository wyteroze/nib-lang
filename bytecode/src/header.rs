// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{Read, Write};

pub struct BytecodeHeader {
    pub signature: [u8; 3], // "NBC" (Nib bytecode)
    pub version_major: u8,
    pub version_minor: u8,
}

impl Debug for BytecodeHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap_or("Invalid UTF8 signature");

        write!(f, "Signature: {},", signature)?;
        write!(f, " Version: {}.{},", self.version_major, self.version_minor)?;
        Ok(())
    }
}

impl BytecodeHeader {
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.signature)?; // Signature
        writer.write_all(&[self.version_major, self.version_minor])?; // Version

        Ok(())
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let mut signature = [0u8; 3];
        let mut version_major = [0u8; 1];
        let mut version_minor = [0u8; 1];

        reader.read_exact(&mut signature)?;
        reader.read_exact(&mut version_major)?;
        reader.read_exact(&mut version_minor)?;

        Ok(Self {
            signature,
            version_major: u8::from_le_bytes(version_major),
            version_minor: u8::from_le_bytes(version_minor),
        })
    }
}
