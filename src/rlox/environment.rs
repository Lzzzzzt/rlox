use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    error::{LoxError, Result},
    token::Token,
    types::Literal,
};


pub type Env = Rc<RefCell<HashMap<Rc<String>, Literal>>>;

pub struct Scopes {
    scopes: Vec<Env>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: vec![Rc::new(RefCell::new(HashMap::new()))],
        }
    }

    pub fn define(&mut self, name: Rc<String>, value: Literal) {
        self.scopes
            .last_mut()
            .unwrap()
            .borrow_mut()
            .insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
        if self
            .scopes
            .last()
            .unwrap()
            .borrow()
            .contains_key(&name.lexeme)
        {
            self.scopes
                .last_mut()
                .unwrap()
                .borrow_mut()
                .insert(name.lexeme.clone(), value);
            return Ok(());
        }

        for scope in self.scopes.iter_mut().rev() {
            if scope.borrow().contains_key(&name.lexeme) {
                scope.borrow_mut().insert(name.lexeme.clone(), value);
                return Ok(());
            }
        }

        Err(LoxError::create_runtime_error(
            name,
            format!("Undefine variable '{}'", &name.lexeme),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        if self
            .scopes
            .last()
            .unwrap()
            .borrow()
            .contains_key(&name.lexeme)
        {
            return Ok(self
                .scopes
                .last()
                .unwrap()
                .borrow()
                .get(&name.lexeme)
                .unwrap()
                .clone());
        }

        for scope in self.scopes.iter().rev() {
            if scope.borrow().contains_key(&name.lexeme) {
                return Ok(scope.borrow().get(&name.lexeme).unwrap().clone());
            }
        }

        Err(LoxError::create_runtime_error(
            name,
            format!("Undefine variable '{}'", &name.lexeme),
        ))
    }

    pub fn scope_begin(&mut self) {
        self.scopes.push(Rc::new(RefCell::new(HashMap::new())));
    }

    pub fn scope_end(&mut self) {
        self.scopes.pop();
    }

    pub fn push_scope(&mut self, scope: Env) {
        self.scopes.push(scope);
    }

    pub fn current(&self) -> Env {
        self.scopes.last().unwrap().clone()
    }
}
