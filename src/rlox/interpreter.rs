use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{
    callable::{Callable, CallableMut},
    environment::Scopes,
    error::{LoxError, Result},
    expr::{Expression, Visitor as ExprVisitor},
    stmt::{Statement, Visitor as StmtVisitor},
    token::Token,
    types::{Class, Function, Literal},
    types::{Lambda, TokenType},
};

pub struct Interpreter {
    pub scopes: Rc<RefCell<Scopes>>,
    cache: LRUCache<Literal>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: Rc::new(RefCell::new(Scopes::new())),
            cache: LRUCache::new(),
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
            position: (0, 0),
            lexeme: Rc::new(obj.to_string()),
            msg: "Expected type is `bool`".into(),
        })
    }

    #[inline]
    #[allow(unused)]
    fn get_func(&self, obj: &Literal, op: &Token) -> Result<Function> {
        if let Literal::Func(func) = obj {
            return Ok(func.clone());
        }

        Err(LoxError::create_runtime_error(
            op,
            "Target must be callable.".into(),
        ))
    }

    #[inline]
    #[allow(unused)]
    fn get_lambda(&self, obj: &Literal, op: &Token) -> Result<Lambda> {
        if let Literal::Lambda(lambda) = obj {
            return Ok(lambda.clone());
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
            String(_) | Number(_) | Func(_) | Lambda(_) | Class(_) | Instance(_) => true,
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
            TokenType::Mod => {
                let left = self.get_num(&left, op)? as i64;
                let right = self.get_num(&right, op)? as i64;
                if (right == 0) {
                    return Err(LoxError::create_runtime_error(
                        op,
                        "divisor cannot be 0.".into(),
                    ));
                }
                Ok(Literal::Number((left % right) as f64))
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
        let mut callee = self.evaluate(&call_expression.callee)?;

        match &mut callee {
            Literal::Func(func) => {
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

                let callee_id = format!(
                    "{}({})",
                    func.name,
                    arguments
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );

                if self.cache.contains_key(&callee_id) {
                    return Ok(self.cache.get(&callee_id).unwrap().clone());
                }
                let res = func.call(self, arguments)?;
                self.cache.insert(callee_id, res.clone());
                Ok(res)
            }
            Literal::Lambda(lambda) => {
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

                let callee_id = format!(
                    "{}({})",
                    lambda.unique,
                    arguments
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );

                if self.cache.contains_key(&callee_id) {
                    return Ok(self.cache.get(&callee_id).unwrap().clone());
                }
                let res = lambda.call(self, arguments)?;
                self.cache.insert(callee_id, res.clone());
                Ok(res)
            }
            Literal::Class(class) => {
                let mut arguments = {
                    let mut a = vec![];
                    for arg in &call_expression.arguments {
                        a.push(self.evaluate(arg)?);
                    }
                    a
                };

                if arguments.len() != class.parameter_num() {
                    return Err(LoxError::create_runtime_error(
                        &call_expression.paren,
                        format!(
                            "Expect {} parameters, but got {}",
                            class.parameter_num(),
                            arguments.len()
                        ),
                    ));
                }

                class.call(self, arguments)
            }
            _ => Err(LoxError::create_runtime_error(
                &call_expression.paren,
                "Target must be callable.".into(),
            )),
        }
    }

    fn visit_get_expression(
        &mut self,
        get_expression: &super::expr::GetExpression,
    ) -> Result<Literal> {
        let obj = self.evaluate(&get_expression.object)?;

        if let Literal::Instance(i) = obj {
            i.get(&get_expression.name)
        } else if let Literal::Class(c) = obj {
            c.get_static_method(&get_expression.name)
        } else {
            Err(LoxError::create_runtime_error(
                &get_expression.name,
                "Only instances have property".into(),
            ))
        }
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
        let mut obj = self.evaluate(&set_expression.object)?;

        if let Literal::Instance(mut i) = obj {
            let value = self.evaluate(&set_expression.value)?;
            i.set(&set_expression.name, value.clone());
            return Ok(value);
        }

        Err(LoxError::create_runtime_error(
            &set_expression.name,
            "Only instances can set property".into(),
        ))
    }

    fn visit_super_expression(
        &mut self,
        super_expression: &super::expr::SuperExpression,
    ) -> Result<Literal> {
        todo!()
    }

    fn visit_self_expression(
        &mut self,
        self_expression: &super::expr::SelfExpression,
    ) -> Result<Literal> {
        self.scopes.as_ref().borrow().get(&self_expression.keyword)
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
        self.scopes.borrow_mut().get(&variable_expression.name)
    }

    fn visit_lambda_expression(
        &mut self,
        lambda_expression: &super::expr::LambdaExpression,
    ) -> Result<Literal, LoxError> {
        Ok(Literal::Lambda(Lambda::from_lambda(
            lambda_expression,
            self.scopes.as_ref().borrow().current(),
        )))
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
        if std::env::var("RLOX_RUN_MODE").unwrap() == "R" {
            println!("\x1b[1;34m[REPL]: \x1b[0m{}", value);
        } else {
            println!("{}", value);
        }
        Ok(())
    }

    fn visit_var_statement(&mut self, var_statement: &super::stmt::VarStatement) -> Result<()> {
        if var_statement.initializer.is_some() {
            let value = self.evaluate(var_statement.initializer.as_ref().unwrap())?;
            self.scopes
                .borrow_mut()
                .define(var_statement.name.lexeme.clone(), value)
        } else {
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
        let func = Literal::Func(Function::new(
            function_statement,
            self.scopes.borrow().current(),
            false,
        ));

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

    fn visit_class_statement(
        &mut self,
        class_statement: &super::stmt::ClassStatement,
    ) -> Result<(), LoxError> {
        self.scopes
            .as_ref()
            .borrow_mut()
            .define(class_statement.name.lexeme.clone(), Literal::Nil);
        let mut methods = HashMap::new();

        for method in &class_statement.methods {
            if let Statement::FunctionStatement(method) = method {
                let m = Literal::Func(Function::new(
                    method,
                    self.scopes.borrow().current(),
                    method.name.lexeme.as_ref().eq("__init__"),
                ));
                methods.insert(method.name.lexeme.clone(), m);
            }
        }

        let mut static_methods = HashMap::new();

        for s_methods in &class_statement.static_methods {
            if let Statement::FunctionStatement(method) = s_methods {
                let m = Literal::Func(Function::new(
                    method,
                    self.scopes.borrow().current(),
                    method.name.lexeme.as_ref().eq("__init__"),
                ));
                static_methods.insert(method.name.lexeme.clone(), m);
            }
        }

        let class = Literal::Class(Class::new(
            class_statement.name.lexeme.clone(),
            methods,
            static_methods,
        ));
        self.scopes
            .as_ref()
            .borrow_mut()
            .assign(&class_statement.name, class)?;

        Ok(())
    }
}

#[derive(Debug)]
struct LRUCache<T> {
    cache: HashMap<String, (T, usize)>,
    count: u32,
}

impl<T> LRUCache<T> {
    fn new() -> Self {
        Self {
            count: 0,
            cache: HashMap::with_capacity(512),
        }
    }

    fn get(&mut self, key: &String) -> Option<&T> {
        self.clean(key);

        let value = match self.cache.get_mut(key) {
            Some(value) => {
                self.count += 1;
                value.1 += 1;
                Some(&value.0)
            }
            None => None,
        };

        value
    }

    fn insert(&mut self, key: String, value: T) {
        self.clean(&key);
        self.count += 1;
        self.cache.insert(key, (value, 0));
    }

    fn contains_key(&self, key: &String) -> bool {
        self.cache.contains_key(key)
    }

    fn clean(&mut self, key: &String) {
        if self.count % 10 == 0 && self.cache.len() > 500 {
            let mut used = self.cache.iter().collect::<Vec<(&String, &(T, usize))>>();
            let len = self.cache.len();
            used.sort_by(|a, b| a.1 .1.cmp(&b.1 .1));

            let less_used = used
                .iter()
                .enumerate()
                .filter(|(i, &(k, _))| i < &(len / 10) && k != key)
                .map(|(_, &(k, _))| k.to_string())
                .collect::<HashSet<String>>();

            self.cache.retain(|k, _| !less_used.contains(k))
        }
    }
}
