use std::{collections::HashMap, rc::Rc};

use super::{
    error::LoxError,
    expr::{Expression, Visitor as ExprVisitor},
    stmt::{Statement, Visitor as StmtVisitor},
};

#[allow(unused)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum VariableState {
    Unused { depth: usize },
    Declared { depth: usize },
    Defined { depth: usize },
}
pub struct Resolver {
    state: HashMap<Rc<String>, VariableState>,
    current_depth: usize,
}

#[allow(unused)]
impl Resolver {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            current_depth: 0,
        }
    }

    pub fn resolve(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), LoxError> {
        statement.accept(self)
    }

    fn resolve_statements(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), LoxError> {
        expression.accept(self)
    }

    pub fn define(&mut self, name: &Rc<String>) {
        self.state.insert(
            name.clone(),
            VariableState::Defined {
                depth: self.current_depth,
            },
        );
    }

    pub fn declare(&mut self, name: &Rc<String>) {
        self.state.insert(
            name.clone(),
            VariableState::Declared {
                depth: self.current_depth,
            },
        );
    }

    pub fn scope_begin(&mut self) {
        self.current_depth += 1;
    }

    pub fn scope_end(&mut self) {
        self.current_depth -= 1;
    }
}

impl StmtVisitor<(), LoxError> for Resolver {
    fn visit_var_statement(
        &mut self,
        var_statement: &super::stmt::VarStatement,
    ) -> Result<(), LoxError> {
        self.declare(&var_statement.name.lexeme);
        if let Some(init) = &var_statement.initializer {
            self.resolve_expression(init)?
        }
        self.define(&var_statement.name.lexeme);
        Ok(())
    }

    fn visit_multi_var_statement(
        &mut self,
        multi_var_statement: &super::stmt::MultiVarStatement,
    ) -> Result<(), LoxError> {
        self.resolve_statements(&multi_var_statement.vars)
    }
}

impl ExprVisitor<(), LoxError> for Resolver {}
