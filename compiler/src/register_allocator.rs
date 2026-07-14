// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

#[derive(Clone)]
pub struct RegisterAllocator {
    next_register: u8,
    freed_registers: Vec<u8>,
    scopes: Vec<u8>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self { next_register: 0, freed_registers: Vec::new(), scopes: Vec::new() }
    }

    pub fn alloc(&mut self) -> u8 {
        if let Some(reg) = self.freed_registers.pop() {
            reg
        } else {
            let reg = self.next_register;
            self.next_register = self
                .next_register
                .checked_add(1)
                .expect("register overflow");

            reg
        }
    }

    pub fn dealloc(&mut self, register: u8) {
        if self.next_register - 1 == register {
            self.next_register -= 1;
            self.freed_registers.retain(|r| *r < self.next_register);
        } else {
            self.freed_registers.push(register)
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(self.next_register)
    }

    pub fn pop_scope(&mut self) {
        let old = self.scopes.pop().expect("no scope to pop");

        self.freed_registers.retain(|r| *r < old);
        self.next_register = old;
    }

    #[allow(unused)]
    pub fn frame_size(&self) -> u8 {
        self.next_register
    }
}
