// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use bytecode::chunk::{Chunk, ChunkInfo};
use bytecode::constant::Constant;
use bytecode::file::BytecodeFile;
use bytecode::header::BytecodeHeader;
use bytecode::instruction::Instruction;

pub fn build(instructions: Vec<Instruction>, constants: Vec<Constant>) -> BytecodeFile {
    BytecodeFile {
        header: BytecodeHeader { signature: *b"NBC", version_major: 255, version_minor: 255 },

        top_level: Chunk {
            info: ChunkInfo {
                arity: 0,
                instruction_count: instructions.len() as u16,
                constant_count: constants.len() as u16,
            },
            instructions,
            constants,
        },
    }
}
