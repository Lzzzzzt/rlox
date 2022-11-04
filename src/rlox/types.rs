use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

use super::{
    callable::Callable,
    environment::Environment,
    error::LoxError,
    expr::LambdaExpression,
    stmt::{FunctionStatement, Statement},
    token::Token,
};
#[allow(dead_code)]
pub enum FuncType {
    Normal,
    Method,
}
impl Display for FuncType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FuncType::Normal => "Function",
                FuncType::Method => "Method",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    // 单或双字符标记
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

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
    This,
    True,
    Let,
    While,
    Continue,
    Break,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(Rc<String>),
    Number(f64),
    Bool(bool),
    Func(Rc<Function>),
    Lambda(Rc<Lambda>),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Func(func) => write!(f, "{}", func),
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "Nil"),
            Literal::Lambda(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lambda {
    parameters: Vec<Token>,
    body: Vec<Statement>,
    closure: Rc<RefCell<Environment>>,
}

impl Lambda {
    pub fn from_lambda(lambda: &LambdaExpression, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            parameters: lambda.params.clone(),
            body: lambda.body.clone(),
            closure,
        }
    }

    pub fn from_function(
        declaration: &FunctionStatement,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            parameters: declaration.params.clone(),
            body: declaration.body.clone(),
            closure,
        }
    }
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Lambda({:?})>",
            self.parameters
                .iter()
                .map(|token| token.lexeme.clone().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Callable for Lambda {
    fn call(
        &self,
        interpreter: &mut super::interpreter::Interpreter,
        arguments: Vec<Literal>,
    ) -> super::error::Result<Literal> {
        let mut env = Environment::new(Some(self.closure.clone()));

        for (i, param) in self.parameters.iter().enumerate() {
            env.define(param.lexeme.clone(), arguments.get(i).unwrap().clone());
        }

        if let Err(e) = interpreter.execute_block_statement(&self.body, env.clone()) {
            match e {
                LoxError::Return { value } => Ok(value),
                _ => Err(e),
            }
        } else {
            Ok(Literal::Nil)
        }
    }

    fn parameter_num(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    name: Rc<String>,
    lambda: Lambda,
}

impl Function {
    pub fn new(declaration: &FunctionStatement, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            name: declaration.name.lexeme.clone(),
            lambda: Lambda::from_function(declaration, closure),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.lambda == other.lambda
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Func {}>", self.name)
    }
}

impl Callable for Function {
    fn call(
        &self,
        interpreter: &mut super::interpreter::Interpreter,
        arguments: Vec<Literal>,
    ) -> super::error::Result<Literal> {
        self.lambda.call(interpreter, arguments)
    }

    fn parameter_num(&self) -> usize {
        self.lambda.parameter_num()
    }
}
