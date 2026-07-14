// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::error::CompileError;
use crate::register_allocator::RegisterAllocator;
use bytecode::chunk::{Chunk, ChunkInfo};
use bytecode::constant::Constant;
use bytecode::file::BytecodeFile;
use bytecode::header::BytecodeHeader;
use bytecode::instruction::Instruction;
use bytecode::opcodes::*;
use syntax::lexer::token::Token;
use syntax::parser::expr::Expr;
use syntax::parser::stmt::Stmt;
use std::collections::HashMap;

const VERSION_MAJOR: u8 = 0;
const VERSION_MINOR: u8 = 0;

#[derive(Copy, Clone)]
enum Local {
    Mutable(u8),
    Immutable(u8),
}

#[derive(Clone)]
struct CompilerFrame {
    register_allocator: RegisterAllocator,
    locals: HashMap<String, Local>,
    constants: Vec<Constant>,
    instructions: Vec<Instruction>,
}

impl CompilerFrame {
    pub fn new() -> Self {
        Self {
            register_allocator: RegisterAllocator::new(),
            locals: HashMap::new(),
            constants: Vec::new(),
            instructions: Vec::new(),
        }
    }
}

pub struct Compiler {
    compiler_frames: Vec<CompilerFrame>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self { compiler_frames: Vec::new() }
    }

    fn build_header(&self) -> BytecodeHeader {
        BytecodeHeader { signature: *b"NBC", version_major: VERSION_MAJOR, version_minor: VERSION_MINOR }
    }

    fn frame(&self) -> &CompilerFrame {
        self.compiler_frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CompilerFrame {
        self.compiler_frames.last_mut().unwrap()
    }

    fn build_chunk(&self, compiler_frame: &CompilerFrame, arity: u8) -> Chunk {
        let info = ChunkInfo {
            arity,
            instruction_count: compiler_frame.instructions.len() as u16,
            constant_count: compiler_frame.constants.len() as u16,
        };

        let mut instructions = compiler_frame.instructions.clone();
        let constants = compiler_frame.constants.clone();

        if instructions.last().is_none() || instructions.last().unwrap().opcode != Op::Return {
            instructions.push(Instruction { opcode: Op::ReturnVoid, a: 0, b: 0, c: 0 });
        }

        Chunk { info, instructions, constants }
    }

    fn add_constant(&mut self, constant: Constant) -> Result<u8, CompileError> {
        let frame = self.frame_mut();
        if frame.constants.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }

        frame.constants.push(constant);
        Ok((frame.constants.len() - 1) as u8)
    }

    fn emit(&mut self, opcode: Op, a: u8, b: u8, c: u8) -> usize {
        let frame = self.frame_mut();
        let instr = Instruction { opcode, a, b, c };
        let instr_pos = frame.instructions.len();

        frame.instructions.push(instr);
        instr_pos
    }

    fn register_local(&mut self, name: String, register: u8) -> Result<(), CompileError> {
        let frame = self.frame_mut();
        frame.locals.insert(name, Local::Mutable(register));

        Ok(())
    }

    fn register_local_immutable(&mut self, name: String, register: u8) -> Result<(), CompileError> {
        let frame = self.frame_mut();
        frame.locals.insert(name, Local::Immutable(register));

        Ok(())
    }

    fn get_local(&mut self, name: &String) -> Option<Local> {
        let frame = self.frame_mut();
        frame.locals.get(name).copied()
    }

    pub fn compile(&mut self, ast: Vec<Stmt>) -> Result<BytecodeFile, CompileError> {
        self.compiler_frames.push(CompilerFrame::new()); // top level frame
        self.compile_ast(ast)?;

        let top_level_frame = self.compiler_frames.first().unwrap();
        let top_level = self.build_chunk(top_level_frame, 0);

        Ok(BytecodeFile { header: self.build_header(), top_level })
    }

    fn compile_ast(&mut self, ast: Vec<Stmt>) -> Result<(), CompileError> {
        for stmt in ast {
            self.compile_stmt(stmt)?;
        }

        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Var { name, initializer } => {
                let register = self.frame_mut().register_allocator.alloc();

                if let Some(init) = initializer {
                    let value_register = self.compile_expr(init)?;

                    self.emit(Op::Move, register, value_register, 0);
                    self.frame_mut().register_allocator.dealloc(value_register);
                }

                self.register_local(name.lexeme, register)?;
                Ok(())
            }
            Stmt::Const { name, initializer } => {
                let register = self.frame_mut().register_allocator.alloc();

                let value_register = self.compile_expr(initializer)?;
                self.emit(Op::Move, register, value_register, 0);

                self.register_local_immutable(name.lexeme, register)?;
                Ok(())
            }
            Stmt::Assign { target, value } => {
                if let Expr::Identifier(t) = &target {
                    let local = self
                        .get_local(&t.lexeme)
                        .ok_or(CompileError::UnknownLocal(t.lexeme.clone()))?;

                    if let Local::Mutable(r) = local {
                        let value_register = self.compile_expr(value)?;

                        self.emit(Op::Move, r, value_register, 0);
                        Ok(())
                    } else {
                        Err(CompileError::ImmutableLocal(t.lexeme.clone()))
                    }
                } else {
                    Err(CompileError::InvalidAssignment)
                }
            }
            Stmt::Expr(e) => {
                self.compile_expr(e)?;

                Ok(())
            }
            Stmt::Return { return_value } => {
                if let Some(v) = return_value {
                    let return_value_register = self.compile_expr(v)?;
                    self.emit(Op::Return, return_value_register, 0, 0);
                    self.frame_mut()
                        .register_allocator
                        .dealloc(return_value_register);
                    Ok(())
                } else {
                    self.emit(Op::ReturnVoid, 0, 0, 0);
                    Ok(())
                }
            }
            Stmt::If { condition, if_branch, else_branch } => {
                let condition_register = self.compile_expr(condition)?;

                match else_branch {
                    // `if` only
                    None => {
                        self.frame_mut().register_allocator.push_scope();
                        let cond_jump_pos = self.emit(Op::JumpIfFalsy, 0, 0, condition_register);

                        for stmt in if_branch {
                            self.compile_stmt(stmt)?;
                        }

                        self.patch_jump(cond_jump_pos);
                        self.frame_mut().register_allocator.pop_scope();

                        Ok(())
                    }

                    // `if` and `else`
                    Some(else_branch) => {
                        self.frame_mut().register_allocator.push_scope();
                        let cond_jump_pos = self.emit(Op::JumpIfFalsy, 0, 0, condition_register);

                        for stmt in if_branch {
                            self.compile_stmt(stmt)?;
                        }

                        // jump past else block, emitted at end of if branch
                        let end_jump_pos = self.emit(Op::Jump, 0, 0, 0);

                        self.patch_jump(cond_jump_pos);
                        self.frame_mut().register_allocator.pop_scope();

                        self.frame_mut().register_allocator.push_scope();
                        for stmt in else_branch {
                            self.compile_stmt(stmt)?;
                        }

                        self.patch_jump(end_jump_pos);
                        self.frame_mut().register_allocator.pop_scope();

                        Ok(())
                    }
                }
            }
            Stmt::Func { name, params, body } => {
                let target_register = self.frame_mut().register_allocator.alloc();
                self.register_local_immutable(name.lexeme, target_register)?;

                let chunk = self.compile_func(params, body)?;
                let const_idx = self.add_constant(Constant::Chunk(chunk))?;

                self.emit(Op::LoadConst, target_register, const_idx, 0);
                Ok(())
            }
        }
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<u8, CompileError> {
        match expr {
            Expr::Literal(value) => {
                let const_idx = self.add_constant(value.into())?;
                let dest = self.frame_mut().register_allocator.alloc();

                self.emit(Op::LoadConst, dest, const_idx, 0);
                Ok(dest)
            }

            Expr::Identifier(token) => {
                let local = self
                    .get_local(&token.lexeme)
                    .ok_or(CompileError::UnknownLocal(token.lexeme.clone()))?;

                match local {
                    Local::Immutable(r) => Ok(r),
                    Local::Mutable(r) => Ok(r),
                }
            }

            Expr::Add { left, right } => self.compile_binary_op(*left, *right, Op::Add),
            Expr::Subtract { left, right } => self.compile_binary_op(*left, *right, Op::Subtract),
            Expr::Multiply { left, right } => self.compile_binary_op(*left, *right, Op::Multiply),
            Expr::Divide { left, right } => self.compile_binary_op(*left, *right, Op::Divide),
            Expr::Exponentiation { left, right } => self.compile_binary_op(*left, *right, Op::Exponentiate),
            Expr::GreaterThan { left, right } => self.compile_binary_op(*left, *right, Op::GreaterThan),
            Expr::GreaterEqualThan { left, right } => self.compile_binary_op(*left, *right, Op::GreaterEqualThan),
            Expr::LessThan { left, right } => self.compile_binary_op(*left, *right, Op::LessThan),
            Expr::LessEqualThan { left, right } => self.compile_binary_op(*left, *right, Op::LessEqualThan),
            Expr::Equals { left, right } => self.compile_binary_op(*left, *right, Op::Equals),
            Expr::NotEquals { left, right } => self.compile_binary_op(*left, *right, Op::NotEquals),
            Expr::Modulo { left, right } => self.compile_binary_op(*left, *right, Op::Modulo),
            Expr::Group { expr } => self.compile_expr(*expr),

            Expr::Not { operand } => {
                let source_register = self.compile_expr(*operand)?;
                let destination_register = self.frame_mut().register_allocator.alloc();

                self.emit(Op::LogicalNot, destination_register, source_register, 0);
                self.frame_mut().register_allocator.dealloc(source_register);

                Ok(destination_register)
            }

            Expr::Negate { operand } => {
                let source_register = self.compile_expr(*operand)?;
                let destination_register = self.frame_mut().register_allocator.alloc();

                self.emit(Op::Negate, destination_register, source_register, 0);
                self.frame_mut().register_allocator.dealloc(source_register);

                Ok(destination_register)
            }

            Expr::And { left, right } => {
                let destination_register = self.frame_mut().register_allocator.alloc();
                let left_register = self.compile_expr(*left)?;

                // if left is falsy then short circuit, move left to destination and jump past right
                self.emit(Op::Move, destination_register, left_register, 0);
                let short_circuit = self.emit(Op::JumpIfFalsy, 0, 0, left_register);

                // left was truthy so we evaluate the right and move it to the destination
                let right_reg = self.compile_expr(*right)?;
                self.emit(Op::Move, destination_register, right_reg, 0);

                self.patch_jump(short_circuit);

                self.frame_mut().register_allocator.dealloc(left_register);
                self.frame_mut().register_allocator.dealloc(right_reg);
                Ok(destination_register)
            }

            Expr::Or { left, right } => {
                let destination_register = self.frame_mut().register_allocator.alloc();
                let left_register = self.compile_expr(*left)?;

                self.emit(Op::Move, destination_register, left_register, 0);
                let short_circuit = self.emit(Op::JumpIfTruthy, 0, 0, left_register);

                let right_register = self.compile_expr(*right)?;
                self.emit(Op::Move, destination_register, right_register, 0);

                self.patch_jump(short_circuit);

                self.frame_mut().register_allocator.dealloc(left_register);
                self.frame_mut().register_allocator.dealloc(right_register);
                Ok(destination_register)
            }

            Expr::Func { params, body } => {
                let chunk = self.compile_func(params, body)?;
                let const_idx = self.add_constant(Constant::Chunk(chunk))?;
                let dest = self.frame_mut().register_allocator.alloc();

                self.emit(Op::LoadConst, dest, const_idx, 0);
                Ok(dest)
            }

            Expr::Call { callee, args } => {
                let callee_local = self.compile_expr(*callee)?;
                let arity = args.len();
                let dest = self.frame_mut().register_allocator.alloc();

                for (i, arg) in args.into_iter().enumerate() {
                    let reg = self.compile_expr(arg)?;
                    self.emit(Op::Move, dest + 1 + i as u8, reg, 0);
                }

                self.emit(Op::Call, dest, callee_local, arity as u8);
                self.frame_mut().register_allocator.dealloc(callee_local);
                Ok(dest)
            }
        }
    }

    fn compile_binary_op(&mut self, left: Expr, right: Expr, op: Op) -> Result<u8, CompileError> {
        let left = self.compile_expr(left)?;
        let right = self.compile_expr(right)?;
        let dest = self.frame_mut().register_allocator.alloc();

        self.emit(op, dest, left, right);
        self.safe_dealloc(left);
        self.safe_dealloc(right);

        Ok(dest)
    }

    // Only deallocates if the register isn't a local (aka, is a temporary)
    fn safe_dealloc(&mut self, register: u8) {
        let frame = self.frame();
        let is_local = frame.locals.values().any(|l| match l {
            Local::Mutable(l) if register == *l => true,
            Local::Immutable(l) if register == *l => true,

            _ => false,
        });

        if !is_local {
            self.frame_mut().register_allocator.dealloc(register)
        }
    }

    fn patch_jump(&mut self, jump_pos: usize) {
        // subtract one since pc will be pointing past jump instruction
        let offset = (self.frame().instructions.len() - jump_pos - 1) as i16;
        let [a, b] = offset.to_le_bytes();
        let inst = self.frame_mut().instructions.get_mut(jump_pos).unwrap();

        inst.a = a;
        inst.b = b;
    }

    fn compile_func(&mut self, params: Vec<Token>, body: Vec<Stmt>) -> Result<Chunk, CompileError> {
        self.compiler_frames.push(CompilerFrame::new());
        let arity = params.len();
        for param in params {
            let register = self.frame_mut().register_allocator.alloc();
            self.register_local(param.lexeme, register)?;
        }

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        let frame = self.compiler_frames.pop().unwrap();
        let chunk = self.build_chunk(&frame, arity as u8);

        Ok(chunk)
    }
}
