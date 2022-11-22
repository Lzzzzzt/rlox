use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use super::{
    bytecode_interpreter::chunk::Chunk,
    error::{LoxError, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncType {
    Main,
    Normal,
    Method,
    Lambda,
    StaticMethod,
    Initializer,
}
impl Display for FuncType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FuncType::Normal => "Function",
                FuncType::Method | FuncType::Initializer => "Method",
                FuncType::Lambda => "Lambda",
                FuncType::Main => "",
                FuncType::StaticMethod => "StaticMethod",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    None,
    Class,
}

impl Display for ClassType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassType::None => write!(f, ""),
            ClassType::Class => write!(f, "Class"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // 单字符标记
    Colon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    QuestionMark,
    Semicolon,
    Slash,
    Star,
    Mod,

    // 单或双字符标记
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    ModEqual,

    // Literals
    Identifier,
    String,
    Number,

    // 关键字
    And,
    Class,
    Else,
    False,
    Func,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    RSelf,
    True,
    Let,
    While,
    Continue,
    Break,
    Static,
    Extend,

    Eof,
}

macro_rules! to_literal {
    ($(($name: ty, $literal_type: ident)), *) => {
        $(
            impl From<$name> for Literal {
                fn from(value: $name) -> Self {
                    Self::$literal_type(value)
                }
            }
        ) *
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(Rc<String>),
    Number(f64),
    Bool(bool),
    Function(Rc<Function>),
    Nil,
}

impl Literal {
    pub fn get_num(&self) -> Result<f64> {
        if let Literal::Number(num) = self {
            return Ok(*num);
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a number!".into(),
        })
    }

    pub fn is_num(&self) -> bool {
        if let Literal::Number(_) = self {
            return true;
        }

        false
    }

    pub fn get_string(&self) -> Result<Rc<String>> {
        if let Literal::String(string) = self {
            return Ok(string.clone());
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a string!".into(),
        })
    }

    pub fn is_string(&self) -> bool {
        if let Literal::String(_) = self {
            return true;
        }

        false
    }

    #[allow(unused)]
    pub fn get_bool(&self) -> Result<bool> {
        if let Literal::Bool(b) = self {
            return Ok(*b);
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a bool!".into(),
        })
    }

    #[allow(unused)]
    pub fn is_bool(&self) -> bool {
        if let Literal::Bool(_) = self {
            return true;
        }

        false
    }

    pub fn is_true(&self) -> bool {
        match self {
            Literal::String(_) | Literal::Number(_) | Literal::Function(_) => true,
            Literal::Bool(b) => *b,
            Literal::Nil => false,
        }
    }

    pub fn get_function(&self) -> Result<Rc<Function>> {
        if let Literal::Function(func) = self {
            return Ok(func.clone());
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a callable type(Function, Lambda, Class)!".into(),
        })
    }
}

to_literal! {
    (bool, Bool),
    (f64, Number),
    (Rc<String>, String),
    (Rc<Function>, Function)
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
            Literal::Function(func) => {
                if func.func_type == FuncType::Lambda {
                    write!(f, "<func Lambda>")
                } else {
                    write!(f, "<func {}>", func.name)
                }
            }
        }
    }
}

static mut LAMBDA_ID: u32 = 0;

fn gen_lambda_id() -> String {
    let id = unsafe {
        LAMBDA_ID += 1;
        LAMBDA_ID
    };
    format!("$-{}", id)
}

#[derive(Debug)]
pub struct Function {
    pub name: Rc<String>,
    pub chunk: Chunk,
    pub arity: usize,
    pub func_type: FuncType,
}

impl Function {
    pub fn new(name: Rc<String>, chunk: Chunk, arity: usize, func_type: FuncType) -> Self {
        Self {
            name,
            chunk,
            arity,
            func_type,
        }
    }

    pub fn lambda_name() -> Rc<String> {
        Rc::new(gen_lambda_id())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}( .. )", self.name)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}
