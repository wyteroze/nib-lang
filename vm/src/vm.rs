// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::heap_obj::HeapObj;
use crate::runtime_error::RuntimeError;
use crate::value::Value;
use bytecode::constant::Constant;
use bytecode::file::BytecodeFile;
use bytecode::instruction::Instruction;
use bytecode::opcodes::Op;

pub struct CallFrame {
    register_offset: usize,
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
    ip: usize,
}

pub struct VM {
    call_stack: Vec<CallFrame>,
    registers: Vec<Value>,
    heap: Vec<HeapObj>,
    returned: bool,
}

impl VM {
    pub fn new(bytecode_file: BytecodeFile) -> Self {
        let registers = vec![Value::Void; 256];

        let top_level = bytecode_file.top_level;
        let top_frame = CallFrame {
            register_offset: 0,
            instructions: top_level.instructions,
            constants: top_level.constants,
            ip: 0,
        };

        Self { call_stack: vec![top_frame], registers, heap: Vec::new(), returned: false }
    }

    #[inline]
    pub fn frame(&self) -> &CallFrame {
        self.call_stack.last().unwrap()
    }

    #[inline]
    pub fn frame_mut(&mut self) -> &mut CallFrame {
        self.call_stack.last_mut().unwrap()
    }

    #[inline]
    pub fn get_register(&self, idx: u8) -> Value {
        self.registers[self.frame().register_offset + idx as usize]
    }

    #[inline]
    pub fn set_register(&mut self, idx: u8, val: Value) {
        let register_offset = self.frame().register_offset;
        self.registers[register_offset + idx as usize] = val
    }

    pub fn heap_get(&mut self, heap_idx: usize) -> Option<&HeapObj> {
        self.heap.get(heap_idx)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        loop {
            let inst = {
                let frame = self.frame_mut();
                let i = frame
                    .instructions
                    .get(frame.ip)
                    .ok_or(RuntimeError::IllegalOperation("pc out of bounds".into()))?;

                frame.ip += 1;
                *i
            };

            match inst.opcode {
                Op::LoadConst => self.load_const(inst.a, inst.b)?,
                Op::Move => self.r#move(inst.a, inst.b)?,
                Op::Add => self.add(inst.a, inst.b, inst.c)?,
                Op::Subtract => self.subtract(inst.a, inst.b, inst.c)?,
                Op::Multiply => self.multiply(inst.a, inst.b, inst.c)?,
                Op::Divide => self.divide(inst.a, inst.b, inst.c)?,
                Op::Exponentiate => self.exponentiate(inst.a, inst.b, inst.c)?,
                Op::Jump => self.jump(inst.a, inst.b)?,
                Op::JumpIfTruthy => self.jump_if_truthy(inst.a, inst.b, inst.c)?,
                Op::JumpIfFalsy => self.jump_if_falsy(inst.a, inst.b, inst.c)?,
                Op::LogicalNot => self.logical_not(inst.a, inst.b)?,
                Op::Negate => self.negate(inst.a, inst.b)?,
                Op::GreaterThan => self.greater_than(inst.a, inst.b, inst.c)?,
                Op::GreaterEqualThan => self.greater_equal_than(inst.a, inst.b, inst.c)?,
                Op::LessThan => self.less_than(inst.a, inst.b, inst.c)?,
                Op::LessEqualThan => self.less_equal_than(inst.a, inst.b, inst.c)?,
                Op::Equals => self.equals(inst.a, inst.b, inst.c)?,
                Op::NotEquals => self.not_equals(inst.a, inst.b, inst.c)?,
                Op::Modulo => self.modulo(inst.a, inst.b, inst.c)?,
                Op::Call => self.call(inst.a, inst.b, inst.c)?,

                Op::Return => self.r#return(inst.a)?,
                Op::ReturnVoid => self.return_void()?,
            }

            if self.returned {
                return Ok(self.registers[0]);
            }
        }
    }

    // Opcode handlers

    #[inline]
    fn load_const(&mut self, a: u8, b: u8) -> Result<(), RuntimeError> {
        let frame = self.frame_mut();

        let constant = frame
            .constants
            .get(b as usize)
            .ok_or(RuntimeError::IllegalOperation("invalid constant index".into()))?
            .clone();

        let register_value = match constant {
            Constant::String(s) => {
                let idx = self.heap.len();
                self.heap.push(HeapObj::String(s));

                Value::HeapObj(idx)
            }
            Constant::Chunk(c) => {
                let idx = self.heap.len();
                self.heap.push(HeapObj::Chunk(c));

                Value::HeapObj(idx)
            }

            _ => Value::from(constant),
        };

        self.set_register(a, register_value);

        Ok(())
    }

    #[inline]
    fn r#move(&mut self, a: u8, b: u8) -> Result<(), RuntimeError> {
        let right = self.get_register(b);
        self.set_register(a, right);

        Ok(())
    }

    #[inline]
    fn add(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        match (left, right) {
            (Value::Number(l), Value::Number(r)) => {
                self.set_register(a, Value::Number(l + r));
                Ok(())
            }
            (Value::HeapObj(l_idx), Value::HeapObj(r_idx)) => {
                let concatenated = {
                    let l = self
                        .heap
                        .get(l_idx)
                        .ok_or(RuntimeError::IllegalOperation("invalid heap index".into()))?;
                    let r = self
                        .heap
                        .get(r_idx)
                        .ok_or(RuntimeError::IllegalOperation("invalid heap index".into()))?;

                    match (l, r) {
                        (HeapObj::String(l_str), HeapObj::String(r_str)) => {
                            let mut s = String::with_capacity(l_str.len() + r_str.len());
                            s.push_str(l_str);
                            s.push_str(r_str);

                            s
                        }

                        #[allow(unreachable_patterns)]
                        _ => {
                            return Err(RuntimeError::InvalidOperation(format!("cannot add {} and {}", l, r)));
                        }
                    }
                };

                let idx = self.heap.len();
                self.heap.push(HeapObj::String(concatenated));
                self.set_register(a, Value::HeapObj(idx));
                Ok(())
            }

            _ => Err(RuntimeError::InvalidOperation(format!("cannot add {} and {}", left, right))),
        }
    }

    #[inline]
    fn subtract(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Number(l - r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot subtract {} and {}", left, right)))
    }

    #[inline]
    fn multiply(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Number(l * r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot multiply {} and {}", left, right)))
    }

    #[inline]
    fn divide(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Number(l / r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot divide {} and {}", left, right)))
    }

    #[inline]
    fn exponentiate(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Number(l.powf(r)));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot exponentiate {} and {}", left, right)))
    }

    #[inline]
    fn logical_not(&mut self, a: u8, b: u8) -> Result<(), RuntimeError> {
        let value = self.get_register(b);
        self.set_register(a, Value::Bool(!self.is_truthy(&value)));

        Ok(())
    }

    #[inline]
    fn negate(&mut self, a: u8, b: u8) -> Result<(), RuntimeError> {
        let value = self.get_register(b);

        if let Value::Number(n) = value {
            self.set_register(a, Value::Number(-n));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot negate {}", value)))
    }

    #[inline]
    fn modulo(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Number(l.rem_euclid(r)));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot divide {} and {}", left, right)))
    }

    #[inline]
    fn greater_than(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Bool(l > r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot compare {} > {}", left, right)))
    }

    #[inline]
    fn greater_equal_than(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Bool(l >= r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot compare {} >= {}", left, right)))
    }

    #[inline]
    fn less_than(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Bool(l < r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot compare {} < {}", left, right)))
    }

    #[inline]
    fn less_equal_than(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        if let Value::Number(l) = left
            && let Value::Number(r) = right
        {
            self.set_register(a, Value::Bool(l <= r));
            return Ok(());
        }

        Err(RuntimeError::InvalidOperation(format!("cannot compare {} <= {}", left, right)))
    }

    #[inline]
    fn equals(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        self.set_register(a, Value::Bool(self.values_equal(left, right)?));
        Ok(())
    }

    #[inline]
    fn not_equals(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let left = self.get_register(b);
        let right = self.get_register(c);

        self.set_register(a, Value::Bool(!self.values_equal(left, right)?));
        Ok(())
    }

    #[inline]
    fn jump(&mut self, a: u8, b: u8) -> Result<(), RuntimeError> {
        let frame = self.frame_mut();
        let offset = i16::from_le_bytes([a, b]);
        frame.ip = frame
            .ip
            .checked_add_signed(offset as isize)
            .ok_or(RuntimeError::IllegalOperation("jump out of bounds".into()))?;

        Ok(())
    }

    #[inline]
    fn jump_if_truthy(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let value = self.get_register(c);
        if self.is_truthy(&value) {
            self.jump(a, b)?
        }

        Ok(())
    }

    #[inline]
    fn jump_if_falsy(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let value = self.get_register(c);
        if !self.is_truthy(&value) {
            self.jump(a, b)?
        }

        Ok(())
    }

    #[inline]
    fn call(&mut self, a: u8, b: u8, c: u8) -> Result<(), RuntimeError> {
        let frame = self.frame();
        let chunk = self.get_register(b);

        if let Value::HeapObj(idx) = chunk {
            match &self.heap[idx] {
                HeapObj::Chunk(chk) => {
                    let chk = chk.clone();
                    if chk.info.arity != c {
                        return Err(RuntimeError::IllegalOperation("incorrect arity".into()));
                    }

                    println!("{:#?}", b);
                    println!("{:#?}", chk.instructions);
                    println!("{:#?}", chk.constants);

                    self.call_stack.push(CallFrame {
                        register_offset: frame.register_offset + a as usize + 1,
                        instructions: chk.instructions,
                        constants: chk.constants,
                        ip: 0,
                    })
                }

                _ => return Err(RuntimeError::IllegalOperation("can't call a non-chunk".into())),
            }
        }

        Ok(())
    }

    #[inline]
    fn r#return(&mut self, a: u8) -> Result<(), RuntimeError> {
        let return_val = self.registers[self.frame().register_offset + a as usize];
        let frame = self.call_stack.pop().unwrap();

        if self.call_stack.is_empty() {
            self.registers[0] = return_val;
            self.returned = true;
        } else {
            self.registers[frame.register_offset - 1] = return_val;
        }

        Ok(())
    }

    #[inline]
    fn return_void(&mut self) -> Result<(), RuntimeError> {
        let frame = self.call_stack.pop().unwrap();

        if self.call_stack.is_empty() {
            self.returned = true;
        } else {
            self.registers[frame.register_offset - 1] = Value::Void;
        }

        Ok(())
    }

    #[inline]
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil | Value::Void => false,
            Value::Bool(b) if !b => false,

            _ => true,
        }
    }

    #[inline]
    fn values_equal(&self, left: Value, right: Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::HeapObj(l), Value::HeapObj(r)) => {
                let l_obj = self
                    .heap
                    .get(l)
                    .ok_or(RuntimeError::IllegalOperation("invalid heap index".into()))?;
                let r_obj = self
                    .heap
                    .get(r)
                    .ok_or(RuntimeError::IllegalOperation("invalid heap index".into()))?;

                Ok(l_obj == r_obj)
            }

            (Value::HeapObj(_), _) | (_, Value::HeapObj(_)) => Ok(false),
            _ => Ok(left == right),
        }
    }
}
