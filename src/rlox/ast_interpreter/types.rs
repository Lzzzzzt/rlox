use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Formatter},
    rc::Rc,
};

use super::{
    ast_interpreter::environment::Env,
    ast_interpreter::interpreter::{self, Interpreter},
    callable::{Callable, CallableMut},
    error::{LoxError, Result},
    expr::LambdaExpression,
    stmt::{FunctionStatement, Statement},
    token::Token,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncType {
    None,
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
                FuncType::None => "",
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
    Func(Function),
    Lambda(Lambda),
    Class(Class),
    Instance(Instance),
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
        self.get_num().is_ok()
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
        self.get_string().is_ok()
    }

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
        self.get_bool().is_ok()
    }

    pub fn is_true(&self) -> bool {
        match self {
            Literal::String(_)
            | Literal::Number(_)
            | Literal::Func(_)
            | Literal::Lambda(_)
            | Literal::Class(_)
            | Literal::Instance(_) => true,
            Literal::Bool(b) => *b,
            Literal::Nil => false,
        }
    }

    pub fn get_func(&self) -> Result<Function> {
        if let Literal::Func(f) = self {
            return Ok(f.clone());
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a function!".into(),
        })
    }

    pub fn get_lambda(&self) -> Result<Lambda> {
        if let Literal::Lambda(l) = self {
            return Ok(l.clone());
        }

        Err(LoxError::UnexpectedError {
            message: "Expect a lambda!".into(),
        })
    }
}

to_literal!(
    (f64, Number),
    (Rc<String>, String),
    (bool, Bool),
    (Function, Func),
    (Lambda, Lambda),
    (Class, Class),
    (Instance, Instance)
);

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Func(func) => write!(f, "{}", func),
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
            Literal::Lambda(l) => write!(f, "{}", l),
            Literal::Class(c) => write!(f, "{}", c),
            Literal::Instance(i) => write!(f, "{}", i),
        }
    }
}

static mut LAMBDA_COUNT: u32 = 0;

fn define_lambda() -> u32 {
    let count = unsafe { LAMBDA_COUNT };

    unsafe {
        LAMBDA_COUNT += 1;
    }

    count
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub parameters: Rc<Vec<Token>>,
    pub body: Rc<Vec<Statement>>,
    pub unique: u32,
    pub closure: Env,
}

impl Lambda {
    pub fn from_lambda(lambda: &LambdaExpression, closure: Env) -> Self {
        Self {
            parameters: Rc::new(lambda.params.clone()),
            body: Rc::new(lambda.body.clone()),
            unique: define_lambda(),
            closure,
        }
    }

    pub fn from_function(declaration: &FunctionStatement, closure: Env) -> Self {
        Self {
            parameters: Rc::new(declaration.params.clone()),
            body: Rc::new(declaration.body.clone()),
            unique: define_lambda(),
            closure,
        }
    }
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.unique == other.unique
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Lambda({})>",
            self.parameters
                .iter()
                .map(|token| token.lexeme.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Callable for Lambda {
    fn call(
        &self,
        interpreter: &mut interpreter::Interpreter,
        arguments: Vec<Literal>,
    ) -> super::error::Result<Literal> {
        // let mut env = Environment::new(Some(self.closure.clone()));

        interpreter
            .scopes
            .as_ref()
            .borrow_mut()
            .push_scope(self.closure.clone());
        interpreter.scopes.as_ref().borrow_mut().scope_begin();

        for (i, param) in self.parameters.iter().enumerate() {
            interpreter
                .scopes
                .as_ref()
                .borrow_mut()
                .define(param.lexeme.clone(), arguments.get(i).unwrap().clone())
                .unwrap();
        }

        let return_value = if let Err(e) = interpreter.execute_block_statement(&self.body) {
            match e {
                LoxError::Return { value } => Ok(value),
                _ => Err(e),
            }
        } else {
            Ok(Literal::Nil)
        };

        interpreter.scopes.as_ref().borrow_mut().scope_end();
        interpreter.scopes.as_ref().borrow_mut().scope_end();

        return_value
    }

    fn parameter_num(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: Rc<String>,
    lambda: Lambda,
    is_initializer: bool,
}

impl Function {
    pub fn new(declaration: &FunctionStatement, closure: Env, is_initializer: bool) -> Self {
        Self {
            name: declaration.name.lexeme.clone(),
            lambda: Lambda::from_function(declaration, closure),
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Instance) -> Self {
        let env = Rc::new(RefCell::new(self.lambda.closure.as_ref().borrow().clone()));
        env.as_ref()
            .borrow_mut()
            .insert(Rc::new(String::from("self")), Literal::Instance(instance));

        let mut lambda = self.lambda.clone();
        lambda.closure = env;

        Self {
            name: self.name.clone(),
            lambda,
            is_initializer: self.is_initializer,
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
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> super::error::Result<Literal> {
        if self.is_initializer {
            self.lambda.call(interpreter, arguments)?;
            Ok(self
                .lambda
                .closure
                .as_ref()
                .borrow()
                .get(&Rc::new(String::from("self")))
                .unwrap()
                .clone())
        } else {
            self.lambda.call(interpreter, arguments)
        }
    }

    fn parameter_num(&self) -> usize {
        self.lambda.parameter_num()
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    name: Rc<String>,
    methods: Rc<RefCell<HashMap<Rc<String>, Literal>>>,
    static_methods: Rc<RefCell<HashMap<Rc<String>, Literal>>>,
    instance_num: Rc<RefCell<u32>>,
}

impl Class {
    pub fn new(
        name: Rc<String>,
        methods: HashMap<Rc<String>, Literal>,
        static_methods: HashMap<Rc<String>, Literal>,
    ) -> Self {
        Self {
            name,
            methods: Rc::new(RefCell::new(methods)),
            instance_num: Rc::new(RefCell::new(0)),
            static_methods: Rc::new(RefCell::new(static_methods)),
        }
    }

    pub fn get_static_method(&self, name: &Token) -> Result<Literal> {
        match self.static_methods.as_ref().borrow().get(&name.lexeme) {
            Some(value) => {
                if let Literal::Func(f) = value {
                    Ok(Literal::Func(f.clone()))
                } else {
                    Err(LoxError::create_runtime_error(
                        name,
                        format!("Undefined static method '{}'", name.lexeme),
                    ))
                }
            }
            None => Err(LoxError::create_runtime_error(
                name,
                format!("Undefined static method '{}'", name.lexeme),
            )),
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Class {}>", self.name)
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl CallableMut for Class {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> super::error::Result<Literal> {
        let instance = Instance::new(self.clone(), *self.instance_num.as_ref().borrow());
        *self.instance_num.as_ref().borrow_mut() += 1;

        let init_fn_id = Rc::new(String::from("__init__"));

        if self.methods.as_ref().borrow().contains_key(&init_fn_id) {
            let methods = self.methods.as_ref().borrow();
            let init = methods.get(&init_fn_id).unwrap();
            if let Literal::Func(init_fn) = init {
                return init_fn.bind(instance).call(interpreter, arguments);
            }
        }

        Ok(Literal::Instance(instance))
    }

    fn parameter_num(&self) -> usize {
        let init_fn_id = Rc::new(String::from("__init__"));

        if self.methods.as_ref().borrow().contains_key(&init_fn_id) {
            let methods = self.methods.as_ref().borrow();
            let init = methods.get(&init_fn_id).unwrap();
            if let Literal::Func(init_fn) = init {
                return init_fn.parameter_num();
            }
        }

        0
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    id: u32,
    class: Class,
    attribute: Rc<RefCell<HashMap<Rc<String>, Literal>>>,
}

impl Instance {
    pub fn new(class: Class, id: u32) -> Self {
        Self {
            class,
            id,
            attribute: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        match self.attribute.as_ref().borrow_mut().get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match self.class.methods.as_ref().borrow().get(&name.lexeme) {
                Some(value) => {
                    if let Literal::Func(f) = value {
                        Ok(Literal::Func(f.bind(self.clone())))
                    } else {
                        Err(LoxError::create_runtime_error(
                            name,
                            format!("Undefined method '{}'", name.lexeme),
                        ))
                    }
                }
                None => Err(LoxError::create_runtime_error(
                    name,
                    format!("Undefined property '{}'", name.lexeme),
                )),
            },
        }
    }

    pub fn set(&mut self, name: &Token, value: Literal) {
        self.attribute
            .as_ref()
            .borrow_mut()
            .insert(name.lexeme.clone(), value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Instance of `{}`>", self.class)
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.class == other.class
    }
}
