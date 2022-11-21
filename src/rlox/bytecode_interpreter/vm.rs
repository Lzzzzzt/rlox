use std::{collections::HashMap, rc::Rc};

use crate::rlox::{
    error::LoxError,
    types::{FuncType, Function, Literal},
};

use super::opcode::OpCode;

pub struct VirtualMachine {
    stack: Vec<Literal>,
    globals: HashMap<Rc<String>, Literal>,
    is_repl: bool,
    frames: Vec<CallFrame>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            frames: Default::default(),
            is_repl: std::env::var("RLOX_RUN_MODE").unwrap() == "R",
            stack: Vec::with_capacity(256),
            globals: HashMap::with_capacity(256),
        }
    }

    fn read_opcode<'a>(&self, frame: &'a mut CallFrame) -> Result<&'a OpCode, LoxError> {
        let opcode = match frame.function.chunk.get(frame.ip) {
            Some(opcode) => Ok(opcode),
            None => Err(LoxError::UnexpectedError {
                message: String::from("EOF!"),
            }),
        }?;
        frame.ip += 1;

        Ok(opcode)
    }

    fn pop(&mut self) -> Literal {
        self.stack.pop().expect("Stack is already empty!")
    }

    fn push(&mut self, value: Literal) {
        self.stack.push(value)
    }

    fn stack_top_clone(&self) -> Literal {
        self.stack[self.stack.len() - 1].clone()
    }

    fn stack_top_ref(&self) -> &Literal {
        self.stack.last().unwrap()
    }

    fn stack_top_mut(&mut self) -> &mut Literal {
        self.stack.last_mut().unwrap()
    }

    fn stack_nth(&self, i: usize) -> &Literal {
        &self.stack[self.stack.len() - i - 1]
    }

    fn binary_add(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            *self.stack_top_mut() = (left + right).into();
        } else if self.stack_nth(1).is_string() && self.stack_nth(0).is_string() {
            let right = self.pop().get_num().unwrap().to_string();
            let left = self.stack_top_ref().get_string().unwrap().to_string();
            *self.stack_top_mut() = Rc::new(left + &right).into();
        } else if self.stack_nth(1).is_string() {
            let right = self.pop().to_string();
            let left = self.stack_top_ref().get_string().unwrap().to_string();
            *self.stack_top_mut() = Rc::new(left + &right).into();
        } else {
            return Err("Operands must be two numbers or two strings.");
        }
        Ok(())
    }

    fn binary_sub(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            *self.stack_top_mut() = (left - right).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    fn binary_multi(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            *self.stack_top_mut() = (left * right).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    fn binary_div(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            if right == 0.0 {
                return Err("divisor cannot be 0.");
            }
            *self.stack_top_mut() = (left / right).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    fn binary_mod(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap() as i64;
            let left = self.stack_top_ref().get_num().unwrap() as i64;
            if right == 0 {
                return Err("divisor cannot be 0.");
            }
            *self.stack_top_mut() = ((left % right) as f64).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    fn binary_eq(&mut self) {
        let right = &self.pop();
        let left = self.stack_top_ref();
        *self.stack_top_mut() = (left == right).into();
    }

    fn binary_less(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            *self.stack_top_mut() = (left < right).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    fn binary_greater(&mut self) -> Result<(), &'static str> {
        if self.stack_nth(1).is_num() && self.stack_nth(0).is_num() {
            let right = self.pop().get_num().unwrap();
            let left = self.stack_top_ref().get_num().unwrap();
            *self.stack_top_mut() = (left > right).into();
        } else {
            return Err("Operands must be two numbers");
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), LoxError> {
        let mut frame = self.frames.pop().unwrap();
        let mut base = frame.slot;

        while let Ok(opcode) = self.read_opcode(&mut frame) {
            // sleep(Duration::from_millis(500));
            // println!(
            //     "[{}] --> [{}]",
            //     opcode,
            //     self.stack
            //         .iter()
            //         .map(|v| v.to_string())
            //         .collect::<Vec<String>>()
            //         .join(", ")
            // );

            match opcode {
                OpCode::Load(value) => {
                    let value = value.clone();
                    self.push(value);
                }
                OpCode::Negate => {
                    if self.stack_top_ref().is_num() {
                        let value = -self.stack_top_ref().get_num().unwrap();
                        *self.stack_top_mut() = value.into();
                    } else {
                        return Err(self.create_runtime_error(
                            &frame,
                            "-",
                            "Operator `-`'s Operand must be number!",
                        ));
                    }
                }
                OpCode::Add => self
                    .binary_add()
                    .map_err(|e| self.create_runtime_error(&frame, "+", e))?,
                OpCode::Sub => self
                    .binary_sub()
                    .map_err(|e| self.create_runtime_error(&frame, "-", e))?,
                OpCode::Mul => self
                    .binary_multi()
                    .map_err(|e| self.create_runtime_error(&frame, "*", e))?,
                OpCode::Div => self
                    .binary_div()
                    .map_err(|e| self.create_runtime_error(&frame, "/", e))?,
                OpCode::Mod => self
                    .binary_mod()
                    .map_err(|e| self.create_runtime_error(&frame, "%", e))?,
                OpCode::Return => {
                    let value = self.pop();
                    if self.frames.is_empty() {
                        self.pop();
                        break;
                    }
                    unsafe {
                        self.stack.set_len(frame.slot);
                    }
                    self.push(value);
                    frame = self.frames.pop().unwrap();
                    base = frame.slot;
                }
                OpCode::Not => {
                    let value = !self.stack_top_ref().is_true();
                    *self.stack_top_mut() = value.into();
                }
                OpCode::Eq => self.binary_eq(),
                OpCode::Less => self
                    .binary_less()
                    .map_err(|e| self.create_runtime_error(&frame, "<", e))?,
                OpCode::Greater => self
                    .binary_greater()
                    .map_err(|e| self.create_runtime_error(&frame, ">", e))?,
                OpCode::Print => {
                    let value = self.pop();
                    if self.is_repl {
                        println!("\x1b[1;34m[REPL]: \x1b[0m{}", value);
                    } else {
                        println!("{}", value);
                    }
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal(name) => {
                    let name = name.clone();
                    let value = self.pop();
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal(name) => {
                    let name = name.clone();
                    if self.globals.contains_key(&name) {
                        let value = self.globals.get(&name).unwrap().clone();
                        self.push(value);
                    } else {
                        return Err(self.create_runtime_error(
                            &frame,
                            &name,
                            format!("Undefined variable `{}`.", &name).as_str(),
                        ));
                    }
                }
                OpCode::SetGlobal(name) => {
                    let name = name.clone();
                    if self.globals.contains_key(&name) {
                        let value = self.stack_top_clone();
                        self.globals.entry(name).and_modify(|v| {
                            *v = value;
                        });
                    } else {
                        return Err(self.create_runtime_error(
                            &frame,
                            &name,
                            format!("Undefined variable `{}`.", &name).as_str(),
                        ));
                    }
                }
                OpCode::GetLocal(slot) => {
                    let slot = slot + base;
                    let value = self.stack[slot].clone();
                    self.push(value)
                }
                OpCode::SetLocal(slot) => {
                    let slot = slot + base;
                    let value = self.stack_top_clone();
                    self.stack[slot] = value;
                }
                OpCode::JumpIfFalse(offset) => {
                    let offset = offset;
                    if !self.stack_top_ref().is_true() {
                        let offset = *offset;
                        frame.ip += offset;
                    }
                }
                OpCode::JumpIfTrue(offset) => {
                    let offset = offset;
                    if self.stack_top_ref().is_true() {
                        let offset = *offset;
                        frame.ip += offset;
                    }
                }
                OpCode::Jump(offset) => {
                    let offset = *offset;
                    frame.ip += offset;
                }
                OpCode::JumpForward(offset) => {
                    let offset = *offset;
                    frame.ip -= offset;
                }
                OpCode::Call(arity) => {
                    let arity = *arity;
                    let callee = self.stack_nth(arity).get_function()?;
                    if callee.arity != arity {
                        return Err(self.create_runtime_error(
                            &frame,
                            &callee.name,
                            format!("Expect {} arguments but got {}.", callee.arity, arity)
                                .as_str(),
                        ));
                    }
                    self.frames.push(frame);
                    frame = CallFrame::new(callee, 0, self.stack.len() - arity - 1);
                    base = frame.slot;
                    // todo!()
                }
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, function: Function) -> Result<(), LoxError> {
        let func = Rc::new(function);
        self.push(func.clone().into());
        let frame = CallFrame::new(func, 0, self.stack.len());
        self.frames.push(frame);

        self.run()
    }

    fn create_runtime_error(&mut self, frame: &CallFrame, op: &str, msg: &str) -> LoxError {
        let ip = frame.ip;
        let pos = frame.function.chunk.get_position(ip).unwrap();
        let mut msgs = vec![msg.to_string()];
        for fm in self.frames.iter().rev() {
            let ip = fm.ip;
            let pos = fm.function.chunk.get_position(ip).unwrap();
            if FuncType::Lambda == frame.function.func_type {
                msgs.push(format!("[{:2},{:2}] Error in Lambda", pos.0, pos.1))
            } else {
                msgs.push(format!(
                    "[{:2},{:2}] Error in function `{}`",
                    pos.0, pos.1, fm.function.name
                ))
            }
        }
        LoxError::RuntimeError {
            position: pos,
            lexeme: Rc::new(op.into()),
            msg: msgs.join("\n"),
        }
    }
}

#[derive(Debug)]
struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub slot: usize,
}

impl CallFrame {
    fn new(function: Rc<Function>, ip: usize, slot: usize) -> Self {
        Self { function, ip, slot }
    }
}
