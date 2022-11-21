use std::{fmt::Display, rc::Rc};

use crate::rlox::types::Literal;

#[derive(Debug, Clone)]
pub enum OpCode {
    Return,
    Load(Literal),
    Negate,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Eq,
    Less,
    Greater,

    Print,
    Pop,
    DefineGlobal(Rc<String>),
    GetGlobal(Rc<String>),
    SetGlobal(Rc<String>),
    GetLocal(usize),
    SetLocal(usize),
    Jump(usize),
    JumpForward(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Call(usize),
}

impl From<Literal> for OpCode {
    fn from(value: Literal) -> Self {
        Self::Load(value)
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Return => write!(f, "{:<24}", "RETURN"),
            OpCode::Load(v) => {
                let len = v.to_string().len();
                let load = String::from("LOAD");
                let space = String::from(" ").repeat(24 - 4 - len);
                write!(f, "{}{}", load + &space, v)
            }
            OpCode::Negate => write!(f, "{:<24}", "NEGATE"),
            OpCode::Add => write!(f, "{:<24}", "ADD"),
            OpCode::Sub => write!(f, "{:<24}", "SUB"),
            OpCode::Mul => write!(f, "{:<24}", "MULTI"),
            OpCode::Div => write!(f, "{:<24}", "DIV"),
            OpCode::Mod => write!(f, "{:<24}", "MOD"),
            OpCode::Not => write!(f, "{:<24}", "NOT"),
            OpCode::Eq => write!(f, "{:<24}", "EQUAL"),
            OpCode::Less => write!(f, "{:<24}", "LESS"),
            OpCode::Greater => write!(f, "{:<24}", "GREATER"),
            OpCode::Print => write!(f, "{:<24}", "PRINT"),
            OpCode::Pop => write!(f, "{:<24}", "POP"),
            OpCode::DefineGlobal(v) => write!(f, "{:<15} {:>8}", "DEFINE_GLOBAL", v),
            OpCode::GetGlobal(v) => write!(f, "{:<15} {:>8}", "GET_GLOBAL", v),
            OpCode::SetGlobal(v) => write!(f, "{:<15} {:>8}", "SET_GLOBAL", v),
            OpCode::GetLocal(v) => write!(f, "{:<15} {:>8}", "GET_LOCAL", v),
            OpCode::SetLocal(v) => write!(f, "{:<15} {:>8}", "SET_LOCAL", v),
            OpCode::Jump(v) => write!(f, "{:<15} {:>8}", "JUMP", v),
            OpCode::JumpForward(v) => write!(f, "{:<15} {:>8}", "JUMP_FORWARD", v),
            OpCode::JumpIfTrue(v) => write!(f, "{:<15} {:>8}", "JUMP_IF_TRUE", v),
            OpCode::JumpIfFalse(v) => write!(f, "{:<15} {:>8}", "JUMP_IF_FALSE", v),
            OpCode::Call(v) => write!(f, "{:<15} {:>8}", "CALL", v),
        }
    }
}
