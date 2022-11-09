use std::{cell::RefCell, rc::Rc};

use super::{
    callable::Callable,
    environment::Scopes,
    error::{LoxError, Result},
    expr::{Expression, Visitor as ExprVisitor},
    stmt::{Statement, Visitor as StmtVisitor},
    token::Token,
    types::{Function, Literal},
    types::{Lambda, TokenType},
};

pub struct Interpreter {
    pub scopes: Rc<RefCell<Scopes>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(Scopes::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    #[inline]
    fn evaluate(&mut self, expr: &Expression) -> Result<Literal> {
        expr.accept(self)
    }

    #[inline]
    pub fn execute(&mut self, stmt: &Statement) -> Result<()> {
        stmt.accept(self)
    }

    #[inline]
    fn get_num(&self, obj: &Literal, op: &Token) -> Result<f64> {
        if let Literal::Number(num) = obj {
            return Ok(*num);
        }

        Err(LoxError::create_runtime_error(
            op,
            "Operand must be a number.".into(),
        ))
    }

    #[inline]
    fn get_string(&self, obj: &Literal, op: &Token) -> Result<Rc<String>> {
        if let Literal::String(string) = obj {
            return Ok(string.clone());
        }

        Err(LoxError::create_runtime_error(
            op,
            "Operand must be a string.".into(),
        ))
    }

    #[inline]
    fn get_bool(&self, obj: &Literal) -> Result<bool> {
        if let Literal::Bool(b) = obj {
            return Ok(*b);
        }

        Err(LoxError::RuntimeError {
            line: 0,
            lexeme: Rc::new(obj.to_string()),
            msg: "Expected type is `bool`".into(),
        })
    }

    #[inline]
    fn get_func(&self, obj: &Literal, op: &Token) -> Result<Rc<Function>> {
        if let Literal::Func(func) = obj {
            return Ok(Rc::clone(func));
        }

        Err(LoxError::create_runtime_error(
            op,
            "Target must be callable.".into(),
        ))
    }

    #[inline]
    fn get_lambda(&self, obj: &Literal, op: &Token) -> Result<Rc<Lambda>> {
        if let Literal::Lambda(lambda) = obj {
            return Ok(Rc::clone(lambda));
        }

        Err(LoxError::create_runtime_error(
            op,
            "Target must be callable.".into(),
        ))
    }

    #[inline]
    fn is_true(&self, obj: &Literal) -> bool {
        use Literal::*;
        match obj {
            String(_) | Number(_) | Func(_) | Lambda(_) => true,
            Bool(b) => *b,
            Nil => false,
        }
    }

    pub fn execute_block_statement_with_new_env(&mut self, statements: &[Statement]) -> Result<()> {
        self.scopes.as_ref().borrow_mut().scope_begin();

        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                // self.environment = pre;
                self.scopes.as_ref().borrow_mut().scope_end();
                return Err(e);
            }
        }

        self.scopes.as_ref().borrow_mut().scope_end();

        Ok(())
    }

    pub fn execute_block_statement(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }
}

#[allow(unused)]
impl ExprVisitor<Literal, LoxError> for Interpreter {
    fn visit_assign_expression(
        &mut self,
        assign_expression: &super::expr::AssignExpression,
    ) -> Result<Literal> {
        let value = self.evaluate(&assign_expression.value)?;
        // self.environment
        //     .borrow_mut()
        //     .assign(&assign_expression.name, value.clone())?;

        self.scopes
            .as_ref()
            .borrow_mut()
            .assign(&assign_expression.name, value.clone())?;
        Ok(value)
    }

    fn visit_binary_expression(
        &mut self,
        binary_expression: &super::expr::BinaryExpression,
    ) -> Result<Literal> {
        let left = self.evaluate(&binary_expression.left)?;
        let right = self.evaluate(&binary_expression.right)?;
        let op = &binary_expression.op;

        match op.token_type {
            TokenType::Minus => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Number(left - right))
            }
            TokenType::Slash => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                if (right == (0 as f64)) {
                    return Err(LoxError::create_runtime_error(
                        op,
                        "divisor cannot be 0.".into(),
                    ));
                }
                Ok(Literal::Number(left / right))
            }
            TokenType::Star => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Number(left * right))
            }
            TokenType::Plus => match left {
                Literal::String(left) => {
                    let right = self
                        .get_string(&right, op)
                        .unwrap_or_else(|_| Rc::new(right.to_string()));

                    let str = left.to_string() + &right.to_string();
                    Ok(Literal::String(Rc::new(str)))
                }
                Literal::Number(left) => {
                    let right = self.get_num(&right, op)?;
                    Ok(Literal::Number(left + right))
                }
                _ => Err(LoxError::create_runtime_error(
                    &binary_expression.op,
                    "Operands must be two numbers or two strings.".into(),
                )),
            },
            TokenType::Greater => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left > right))
            }
            TokenType::GreaterEqual => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left >= right))
            }
            TokenType::Less => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left < right))
            }
            TokenType::LessEqual => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left <= right))
            }
            TokenType::EqualEqual => Ok(Literal::Bool(left == right)),
            TokenType::BangEqual => Ok(Literal::Bool(left != right)),
            TokenType::Comma => Ok(binary_expression.right.accept(self)?),
            _ => Err(LoxError::create_runtime_error(
                &binary_expression.op,
                "Unexpected operator".into(),
            )),
        }
    }

    fn visit_call_expression(
        &mut self,
        call_expression: &super::expr::CallExpression,
    ) -> Result<Literal> {
        let callee = self.evaluate(&call_expression.callee)?;

        match self.get_func(&callee, &call_expression.paren) {
            Ok(func) => {
                let mut arguments = {
                    let mut a = vec![];
                    for arg in &call_expression.arguments {
                        a.push(self.evaluate(arg)?);
                    }
                    a
                };

                if arguments.len() != func.parameter_num() {
                    return Err(LoxError::create_runtime_error(
                        &call_expression.paren,
                        format!(
                            "Expect {} parameters, but got {}",
                            func.parameter_num(),
                            arguments.len()
                        ),
                    ));
                }

                func.call(self, arguments)
            }
            Err(e) => {
                let mut lambda = self.get_lambda(&callee, &call_expression.paren)?;

                let mut arguments = {
                    let mut a = vec![];
                    for arg in &call_expression.arguments {
                        a.push(self.evaluate(arg)?);
                    }
                    a
                };

                if arguments.len() != lambda.parameter_num() {
                    return Err(LoxError::create_runtime_error(
                        &call_expression.paren,
                        format!(
                            "Expect {} parameters, but got {}",
                            lambda.parameter_num(),
                            arguments.len()
                        ),
                    ));
                }

                lambda.call(self, arguments)
            }
        }
    }

    fn visit_get_expression(
        &mut self,
        get_expression: &super::expr::GetExpression,
    ) -> Result<Literal> {
        todo!()
    }

    fn visit_grouping_expression(
        &mut self,
        grouping_expression: &super::expr::GroupingExpression,
    ) -> Result<Literal> {
        self.evaluate(&grouping_expression.expression)
    }

    fn visit_literal_expression(
        &mut self,
        literal_expression: &super::expr::LiteralExpression,
    ) -> Result<Literal> {
        Ok(literal_expression.value.clone())
    }

    fn visit_logical_expression(
        &mut self,
        logical_expression: &super::expr::LogicalExpression,
    ) -> Result<Literal> {
        let left = self.evaluate(&logical_expression.left)?;

        if logical_expression.op.token_type == TokenType::Or && self.is_true(&left) {
            return Ok(left);
        }

        self.evaluate(&logical_expression.right)
    }

    fn visit_set_expression(
        &mut self,
        set_expression: &super::expr::SetExpression,
    ) -> Result<Literal> {
        todo!()
    }

    fn visit_super_expression(
        &mut self,
        super_expression: &super::expr::SuperExpression,
    ) -> Result<Literal> {
        todo!()
    }

    fn visit_this_expression(
        &mut self,
        this_expression: &super::expr::ThisExpression,
    ) -> Result<Literal> {
        todo!()
    }

    fn visit_ternary_expression(
        &mut self,
        ternary_expression: &super::expr::TernaryExpression,
    ) -> Result<Literal> {
        let value = &self.evaluate(&ternary_expression.cmp)?;
        let cmp = self.get_bool(value)?;

        if cmp {
            Ok(self.evaluate(&ternary_expression.true_value)?)
        } else {
            Ok(self.evaluate(&ternary_expression.false_value)?)
        }
    }

    fn visit_unary_expression(
        &mut self,
        unary_expression: &super::expr::UnaryExpression,
    ) -> Result<Literal> {
        let right = self.evaluate(&unary_expression.right)?;
        let op = &unary_expression.op;

        match op.token_type {
            TokenType::Plus => Ok(Literal::Number(self.get_num(&right, op)?.abs())),
            TokenType::Minus => Ok(Literal::Number(-self.get_num(&right, op)?)),
            TokenType::Bang => Ok(Literal::Bool(!self.is_true(&right))),
            _ => Err(LoxError::create_runtime_error(
                &unary_expression.op,
                "Operand must be number or bool".into(),
            )),
        }
    }

    fn visit_variable_expression(
        &mut self,
        variable_expression: &super::expr::VariableExpression,
    ) -> Result<Literal> {
        // self.environment.borrow_mut().get(&variable_expression.name)
        self.scopes.borrow_mut().get(&variable_expression.name)
    }

    fn visit_lambda_expression(
        &mut self,
        lambda_expression: &super::expr::LambdaExpression,
    ) -> Result<Literal, LoxError> {
        Ok(Literal::Lambda(Rc::new(Lambda::from_lambda(
            lambda_expression,
            // self.environment.clone(),
            self.scopes.as_ref().borrow().current(),
        ))))
    }
}

impl StmtVisitor<(), LoxError> for Interpreter {
    fn visit_expression_statement(
        &mut self,
        expression_statement: &super::stmt::ExpressionStatement,
    ) -> Result<()> {
        self.evaluate(&expression_statement.expression)?;
        Ok(())
    }

    fn visit_print_statement(
        &mut self,
        print_statement: &super::stmt::PrintStatement,
    ) -> Result<()> {
        let value = self.evaluate(&print_statement.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_statement(&mut self, var_statement: &super::stmt::VarStatement) -> Result<()> {
        if var_statement.initializer.is_some() {
            let value = self.evaluate(var_statement.initializer.as_ref().unwrap())?;
            // self.environment
            //     .borrow_mut()
            //     .define(var_statement.name.lexeme.clone(), value)
            self.scopes
                .borrow_mut()
                .define(var_statement.name.lexeme.clone(), value)
        } else {
            // self.environment
            //     .borrow_mut()
            //     .define(var_statement.name.lexeme.clone(), Literal::Nil)
            self.scopes
                .borrow_mut()
                .define(var_statement.name.lexeme.clone(), Literal::Nil)
        }

        Ok(())
    }

    fn visit_multi_var_statement(
        &mut self,
        multi_var_statement: &super::stmt::MultiVarStatement,
    ) -> Result<()> {
        for var in &multi_var_statement.vars {
            self.execute(var)?;
        }
        Ok(())
    }

    fn visit_block_statement(
        &mut self,
        block_statement: &super::stmt::BlockStatement,
    ) -> Result<()> {
        self.execute_block_statement_with_new_env(
            &block_statement.statements,
            // Environment::new(Some(self.environment.clone())),
        )
    }

    fn visit_branch_statement(
        &mut self,
        branch_statement: &super::stmt::BranchStatement,
    ) -> Result<()> {
        let condition = self.evaluate(&branch_statement.condition)?;
        if self.is_true(&condition) {
            self.execute(&branch_statement.then_branch)?
        } else if let Some(es) = branch_statement.else_branch.as_ref() {
            self.execute(es)?
        }

        Ok(())
    }

    fn visit_while_statement(
        &mut self,
        while_statement: &super::stmt::WhileStatement,
    ) -> Result<()> {
        let mut condition = self.evaluate(&while_statement.condition)?;

        while self.is_true(&condition) {
            if let Err(e) = self.execute(&while_statement.body) {
                match e {
                    LoxError::Break { .. } => break,
                    LoxError::Continue { .. } => {
                        if let Some(incr) = &while_statement.increment {
                            self.execute(incr)?;
                        }
                        continue;
                    }
                    _ => return Err(e),
                };
            }
            condition = self.evaluate(&while_statement.condition)?;
        }

        Ok(())
    }

    fn visit_continue_statement(
        &mut self,
        continue_statement: &super::stmt::ContinueStatement,
    ) -> Result<()> {
        Err(LoxError::create_continue(
            &continue_statement.token,
            "'continue' must be in 'for' or 'while' statement".into(),
        ))
    }

    fn visit_break_statement(
        &mut self,
        break_statement: &super::stmt::BreakStatement,
    ) -> Result<()> {
        Err(LoxError::create_break(
            &break_statement.token,
            "'break' must be in 'for' or 'while' statement".into(),
        ))
    }

    fn visit_function_statement(
        &mut self,
        function_statement: &super::stmt::FunctionStatement,
    ) -> Result<(), LoxError> {
        let func = Literal::Func(Rc::new(Function::new(
            function_statement,
            // self.environment.clone(),
            self.scopes.borrow().current(),
        )));

        // self.environment
        //     .borrow_mut()
        //     .define(function_statement.name.lexeme.clone(), func);
        self.scopes
            .borrow_mut()
            .define(function_statement.name.lexeme.clone(), func);

        Ok(())
    }

    fn visit_return_statement(
        &mut self,
        return_statement: &super::stmt::ReturnStatement,
    ) -> Result<(), LoxError> {
        let value = if let Some(v) = &return_statement.value {
            self.evaluate(v)?
        } else {
            Literal::Nil
        };

        Err(LoxError::create_return(value))
    }
}
