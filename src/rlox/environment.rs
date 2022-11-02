use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    error::{LoxError, Result},
    token::{Literal, Token},
};

#[derive(Clone, Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value).unwrap();
            return Ok(());
        }

        if let Some(e) = &self.enclosing {
            return e.borrow_mut().assign(name, value);
        }

        Err(LoxError::create_runtime_error(
            name,
            format!("Undefine variable '{}'", &name.lexeme),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        // println!("scope: {:#?}\n get: {:#?}\n env: {:#?}\n", self.enclosing, name, self.values);
        if self.values.contains_key(&name.lexeme) {
            return Ok(self.values.get(&name.lexeme).unwrap().clone());
        }

        if let Some(e) = &self.enclosing {
            return Ok(e.borrow().get(name).unwrap());
        }

        Err(LoxError::create_runtime_error(
            name,
            format!("Undefine variable '{}'", &name.lexeme),
        ))
    }
}
