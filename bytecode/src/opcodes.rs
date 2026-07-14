// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

/*
Instruction "grammar"

Each "word":
[ instruction (8) ] [ a (8) ] [ b (8) ] [ c (8) ]
each instruction is 32 bits
A, B, C can only be 255 max

x can be A, B, C
reg(x) = register at X
const(x) = constant at X

[NAME] [instruction format] -> [VM operation]

ex.
LOADCONST reg(a) const(b) -> reg(a) = const(b)
human version:

the LOADCONST instruction uses A, an index into a VM register and B, an index into the VM constant pool.
The VM sets the register at register A to the constant at B in the constant pool.

Update instruction.rs if you add a new instruction
*/

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Op {
    /// Loads constant at B into register at A<br>
    /// reg(a) const(b) -> reg(a) = const(b)
    LoadConst = 0,

    /// Moves register A's value to register B<br>
    /// reg(a) reg(b) -> reg(a) = reg(b)
    Move = 1,

    /// Returns value in register A
    Return = 2,

    /// Returns void. This is fundamentally different from returning nil
    /// as it means **nothing** was returned instead of nil being returned.
    ReturnVoid = 3,

    /// Sets register A to the sum of register B + register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) + reg(c)
    Add = 4,

    /// Sets register A to the difference between register B - register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) - reg(c)
    Subtract = 5,

    /// Sets register A to the product between register B * register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) * reg(c)
    Multiply = 6,

    /// Sets register A to the quotient between register B / register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) / reg(c)
    Divide = 7,

    /// Sets register A to the power of register B ^ register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) ^ reg(c)
    Exponentiate = 8,

    /// Adds A and B read as an i16 to the program counter
    /// number(AB) -> ip += number(AB)
    Jump = 9,

    /// Adds A and B read as an i16 to the program counter if the value in register C is truthy
    /// number(AB) reg(c) -> ip += number(AB) if reg(c)
    JumpIfTruthy = 10,

    /// Adds A and B read as an i16 to the program counter if the value in register C is falsy
    /// number(AB) reg(c) -> ip += number(AB) if not reg(c)
    JumpIfFalsy = 11,

    /// Sets register A to the logical not of the value in register B
    /// reg(a) reg(b) -> reg(a) = not reg(b)
    LogicalNot = 12,

    /// Sets register A to the negation of the value in register B
    /// reg(a) reg(b) -> reg(a) = -reg(b)
    Negate = 13,

    /// Sets register A to if register B is greater than register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) > reg(c)
    GreaterThan = 14,

    /// Sets register A to if register B is greater than or equal to register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) >= reg(c)
    GreaterEqualThan = 15,

    /// Sets register A to if register B is less than register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) < reg(c)
    LessThan = 16,

    /// Sets register A to if register B is less than or equal to register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) <= reg(c)
    LessEqualThan = 17,

    /// Sets register A to if register B is equal to register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) == reg(c)
    Equals = 18,

    /// Sets register A to if register B is not equal to register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) != reg(c)
    NotEquals = 19,

    /// Sets register A to the remainder of register B % register C
    /// reg(a) reg(b) reg(c) -> reg(a) = reg(b) % reg(c)
    Modulo = 20,

    /// Calls the chunk at B with C number of params and stores the value at A.
    /// reg(a) reg(b) reg(c0 -> reg(a) = reg(b)(0 to c)
    Call = 21,
}

impl TryFrom<u8> for Op {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Op::LoadConst),
            1 => Ok(Op::Move),
            2 => Ok(Op::Return),
            3 => Ok(Op::ReturnVoid),
            4 => Ok(Op::Add),
            5 => Ok(Op::Subtract),
            6 => Ok(Op::Multiply),
            7 => Ok(Op::Divide),
            8 => Ok(Op::Exponentiate),
            9 => Ok(Op::Jump),
            10 => Ok(Op::JumpIfTruthy),
            11 => Ok(Op::JumpIfFalsy),
            12 => Ok(Op::LogicalNot),
            13 => Ok(Op::Negate),
            14 => Ok(Op::GreaterThan),
            15 => Ok(Op::GreaterEqualThan),
            16 => Ok(Op::LessThan),
            17 => Ok(Op::LessEqualThan),
            18 => Ok(Op::Equals),
            19 => Ok(Op::NotEquals),
            20 => Ok(Op::Modulo),
            21 => Ok(Op::Call),

            _ => Err("Unknown opcode"),
        }
    }
}
