use std::fmt::{Debug, Display};

use super::opcode::OpCode;

pub struct Chunk {
    codes: Vec<OpCode>,
    positions: Vec<(usize, usize)>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            codes: Default::default(),
            positions: Default::default(),
        }
    }

    pub fn write(&mut self, opcode: OpCode, position: (usize, usize)) -> usize {
        self.codes.push(opcode);
        self.positions.push(position);
        self.len() - 1
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&OpCode> {
        self.codes.get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut OpCode> {
        self.codes.get_mut(index)
    }

    #[allow(unused)]
    pub fn get_with_position(&self, index: usize) -> Option<(&OpCode, (usize, usize))> {
        let opcode = self.get(index);
        let line = self.get_position(index);

        if self.get(index).is_some() && self.get_position(index).is_some() {
            Some((opcode.unwrap(), line.unwrap()))
        } else {
            None
        }
    }

    pub fn get_position(&self, index: usize) -> Option<(usize, usize)> {
        self.positions.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.codes.len()
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk").field("codes", &self.codes).finish()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, code) in self.codes.iter().enumerate() {
            writeln!(f, "[{:>4}]: {:?}", i, code)?;
        }
        Ok(())
    }
}
