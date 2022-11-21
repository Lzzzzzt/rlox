#![allow(unused)]

use std::rc::Rc;

use crate::rlox::{
    error::LoxError,
    expr::{Expression, Visitor as ExprVisitor},
    stmt::{Statement, Visitor as StmtVisitor},
    types::{FuncType, Function, Literal, TokenType},
};

use super::{chunk::Chunk, environment::Scopes, opcode::OpCode};

pub struct Convertor {
    function: Function,
    func_type: FuncType,
    scopes: Scopes,
    break_position: Vec<usize>,
    continue_position: Vec<usize>,
    loop_body_depth: usize,
    is_returned: bool,
}

impl Default for Convertor {
    fn default() -> Self {
        Self {
            function: Function::new(Rc::new("__main__".into()), Chunk::new(), 0, FuncType::Main),
            func_type: FuncType::Main,
            scopes: Default::default(),
            break_position: Default::default(),
            continue_position: Default::default(),
            loop_body_depth: Default::default(),
            is_returned: Default::default(),
        }
    }
}

impl Convertor {
    pub fn new(func_name: &str, func_type: FuncType) -> Self {
        let mut scopes: Scopes = Default::default();
        scopes.define_variable(Rc::new(func_name.into()), 0);

        if FuncType::Main != func_type {
            scopes.begin_scope();
        }

        Self {
            function: Function::new(Rc::new(func_name.into()), Chunk::new(), 0, func_type),
            func_type,
            scopes,
            break_position: vec![],
            continue_position: vec![],
            loop_body_depth: 0,
            is_returned: false,
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    pub fn convert(mut self, statements: &[Statement]) -> Result<Function, LoxError> {
        for stmt in statements {
            self.convert_statement(stmt)?;
        }
        // println!("{}", self.current_chunk());
        // println!("{:#?}", self.scopes);

        if !self.is_returned {
            self.current_chunk()
                .write(OpCode::Load(Literal::Nil), (0, 0));
            self.current_chunk().write(OpCode::Return, (0, 0));
        }

        if FuncType::Normal == self.func_type {
            self.scopes.end_scope();
        }

        Ok(self.function)
    }

    fn convert_expression(&mut self, expr: &Expression) -> Result<(), LoxError> {
        expr.accept(self)
    }

    fn convert_statement(&mut self, stmt: &Statement) -> Result<(), LoxError> {
        stmt.accept(self)
    }

    fn convert_statements(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        for stmt in statements {
            stmt.accept(self)?;
        }
        Ok(())
    }

    fn patch_jump_opcode(&mut self, index: usize) {
        let cur = self.current_chunk().len() - 1;
        let code = self.current_chunk().get_mut(index).unwrap();
        match code {
            OpCode::Jump(offset) | OpCode::JumpIfFalse(offset) | OpCode::JumpIfTrue(offset) => {
                *offset = cur - index
            }
            _ => (),
        }
    }

    fn handle_continue_jump(&mut self) {
        if !self.continue_position.is_empty() {
            let cur = self.current_chunk().len() - 1;
            let pos = self.continue_position.pop().unwrap();
            let code = self.current_chunk().get_mut(pos).unwrap();
            if let OpCode::Jump(offset) = code {
                *offset = cur - pos
            };
        }
    }

    fn handle_break_jump(&mut self) {
        if !self.break_position.is_empty() {
            let cur = self.current_chunk().len() - 1;
            let pos = self.continue_position.pop().unwrap();
            let code = self.current_chunk().get_mut(pos).unwrap();
            if let OpCode::Jump(offset) = code {
                *offset = cur - pos
            }
            self.break_position.pop();
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.begin_scope();
    }

    fn end_scope(&mut self) {
        self.scopes.end_scope().into_iter().for_each(|c| {
            self.current_chunk().write(c, (0, 0));
        });
    }
}

impl ExprVisitor<(), LoxError> for Convertor {
    fn visit_assign_expression(
        &mut self,
        assign_expression: &crate::rlox::expr::AssignExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&assign_expression.value)?;

        match self
            .scopes
            .find_variable(assign_expression.name.lexeme.clone())
        {
            Ok(i) => {
                self.function
                    .chunk
                    .write(OpCode::SetLocal(i), assign_expression.name.position);
            }
            Err(_) => {
                self.current_chunk().write(
                    OpCode::SetGlobal(assign_expression.name.lexeme.clone()),
                    assign_expression.name.position,
                );
            }
        }

        Ok(())
    }

    fn visit_binary_expression(
        &mut self,
        binary_expression: &crate::rlox::expr::BinaryExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&binary_expression.left)?;
        self.convert_expression(&binary_expression.right)?;

        let pos = binary_expression.op.position;

        match binary_expression.op.token_type {
            TokenType::Plus => {
                self.function.chunk.write(OpCode::Add, pos);
            }
            TokenType::Minus => {
                self.function.chunk.write(OpCode::Sub, pos);
            }
            TokenType::Star => {
                self.function.chunk.write(OpCode::Mul, pos);
            }
            TokenType::Slash => {
                self.function.chunk.write(OpCode::Div, pos);
            }
            TokenType::Mod => {
                self.function.chunk.write(OpCode::Mod, pos);
            }
            TokenType::BangEqual => {
                self.function.chunk.write(OpCode::Eq, pos);
                self.function.chunk.write(OpCode::Not, pos);
            }
            TokenType::EqualEqual => {
                self.function.chunk.write(OpCode::Eq, pos);
            }
            TokenType::Greater => {
                self.function.chunk.write(OpCode::Greater, pos);
            }
            TokenType::GreaterEqual => {
                self.function.chunk.write(OpCode::Less, pos);
                self.function.chunk.write(OpCode::Not, pos);
            }
            TokenType::Less => {
                self.function.chunk.write(OpCode::Less, pos);
            }
            TokenType::LessEqual => {
                self.function.chunk.write(OpCode::Greater, pos);
                self.function.chunk.write(OpCode::Not, pos);
            }
            _ => {
                return Err(LoxError::create_runtime_error(
                    &binary_expression.op,
                    "Unexpected operator".into(),
                ))
            }
        }
        Ok(())
    }

    fn visit_call_expression(
        &mut self,
        call_expression: &crate::rlox::expr::CallExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&call_expression.callee);
        for arg in &call_expression.arguments {
            self.convert_expression(arg)?;
        }
        self.current_chunk().write(
            OpCode::Call(call_expression.arguments.len()),
            call_expression.paren.position,
        );
        Ok(())
    }

    fn visit_get_expression(
        &mut self,
        get_expression: &crate::rlox::expr::GetExpression,
    ) -> Result<(), LoxError> {
        todo!()
    }

    fn visit_grouping_expression(
        &mut self,
        grouping_expression: &crate::rlox::expr::GroupingExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&grouping_expression.expression)
    }

    fn visit_literal_expression(
        &mut self,
        literal_expression: &crate::rlox::expr::LiteralExpression,
    ) -> Result<(), LoxError> {
        self.current_chunk().write(
            literal_expression.value.clone().into(),
            literal_expression.token.position,
        );
        Ok(())
    }

    fn visit_logical_expression(
        &mut self,
        logical_expression: &crate::rlox::expr::LogicalExpression,
    ) -> Result<(), LoxError> {
        match logical_expression.op.token_type {
            TokenType::And => {
                self.convert_expression(&logical_expression.left)?;
                let index = self
                    .current_chunk()
                    .write(OpCode::JumpIfFalse(0), logical_expression.op.position);
                self.current_chunk().write(OpCode::Pop, (0, 0));
                self.convert_expression(&logical_expression.right)?;
                self.patch_jump_opcode(index);
                Ok(())
            }
            TokenType::Or => {
                self.convert_expression(&logical_expression.left)?;
                let index = self
                    .current_chunk()
                    .write(OpCode::JumpIfTrue(0), logical_expression.op.position);
                self.current_chunk().write(OpCode::Pop, (0, 0));
                self.convert_expression(&logical_expression.right)?;
                self.patch_jump_opcode(index);
                Ok(())
            }
            _ => Err(LoxError::create_runtime_error(
                &logical_expression.op,
                "Unexpected operator".into(),
            )),
        }
    }

    fn visit_set_expression(
        &mut self,
        set_expression: &crate::rlox::expr::SetExpression,
    ) -> Result<(), LoxError> {
        todo!()
    }

    fn visit_super_expression(
        &mut self,
        super_expression: &crate::rlox::expr::SuperExpression,
    ) -> Result<(), LoxError> {
        todo!()
    }

    fn visit_self_expression(
        &mut self,
        self_expression: &crate::rlox::expr::SelfExpression,
    ) -> Result<(), LoxError> {
        todo!()
    }

    fn visit_ternary_expression(
        &mut self,
        ternary_expression: &crate::rlox::expr::TernaryExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&ternary_expression.cmp)?;
        let jump_false = self.current_chunk().write(OpCode::JumpIfFalse(0), (0, 0));
        self.current_chunk().write(OpCode::Pop, (0, 0));
        self.convert_expression(&ternary_expression.true_value)?;
        let jump = self.current_chunk().write(OpCode::Jump(0), (0, 0));
        self.patch_jump_opcode(jump_false);
        self.current_chunk().write(OpCode::Pop, (0, 0));
        self.convert_expression(&ternary_expression.false_value)?;
        self.patch_jump_opcode(jump);

        Ok(())
    }

    fn visit_unary_expression(
        &mut self,
        unary_expression: &crate::rlox::expr::UnaryExpression,
    ) -> Result<(), LoxError> {
        self.convert_expression(&unary_expression.right)?;
        match unary_expression.op.token_type {
            TokenType::Minus => {
                self.function
                    .chunk
                    .write(OpCode::Negate, unary_expression.op.position);
                Ok(())
            }
            TokenType::Bang => {
                self.function
                    .chunk
                    .write(OpCode::Not, unary_expression.op.position);
                Ok(())
            }
            _ => Err(LoxError::create_runtime_error(
                &unary_expression.op,
                "Operand must be number or bool".into(),
            )),
        }
    }

    fn visit_variable_expression(
        &mut self,
        variable_expression: &crate::rlox::expr::VariableExpression,
    ) -> Result<(), LoxError> {
        match self
            .scopes
            .find_variable(variable_expression.name.lexeme.clone())
        {
            Ok(i) => {
                self.function
                    .chunk
                    .write(OpCode::GetLocal(i), variable_expression.name.position);
            }
            Err(_) => {
                self.current_chunk().write(
                    OpCode::GetGlobal(variable_expression.name.lexeme.clone()),
                    variable_expression.name.position,
                );
            }
        }

        Ok(())
    }

    fn visit_lambda_expression(
        &mut self,
        lambda_expression: &crate::rlox::expr::LambdaExpression,
    ) -> Result<(), LoxError> {
        let name = Function::lambda_name();
        let arity = lambda_expression.params.len();
        let mut convertor = Convertor::new(&name, FuncType::Lambda);

        let depth = convertor.scopes.depth;
        for param in &lambda_expression.params {
            convertor
                .scopes
                .define_variable(param.lexeme.clone(), depth);
        }

        let mut func = convertor.convert(&lambda_expression.body)?;

        let func = Rc::new(func);
        self.current_chunk()
            .write(OpCode::Load(func.into()), (0, 0));

        Ok(())
    }
}

impl StmtVisitor<(), LoxError> for Convertor {
    fn visit_expression_statement(
        &mut self,
        expression_statement: &crate::rlox::stmt::ExpressionStatement,
    ) -> Result<(), LoxError> {
        self.convert_expression(&expression_statement.expression)?;
        self.function
            .chunk
            .write(OpCode::Pop, expression_statement.end.position);
        Ok(())
    }

    fn visit_print_statement(
        &mut self,
        print_statement: &crate::rlox::stmt::PrintStatement,
    ) -> Result<(), LoxError> {
        self.convert_expression(&print_statement.expression)?;
        self.function
            .chunk
            .write(OpCode::Print, print_statement.keyword.position);
        Ok(())
    }

    fn visit_var_statement(
        &mut self,
        var_statement: &crate::rlox::stmt::VarStatement,
    ) -> Result<(), LoxError> {
        if let Some(init) = &var_statement.initializer {
            self.convert_expression(init)?;
        } else {
            self.function
                .chunk
                .write(OpCode::Load(Literal::Nil), var_statement.name.position);
        }

        if self.scopes.depth > 0 {
            if self
                .scopes
                .define_variable(var_statement.name.lexeme.clone(), self.scopes.depth)
                .is_err()
            {
                return Err(LoxError::create_runtime_error(
                    &var_statement.name,
                    "Already a variable with this name in this scope.".into(),
                ));
            };
        } else {
            self.current_chunk().write(
                OpCode::DefineGlobal(var_statement.name.lexeme.clone()),
                var_statement.name.position,
            );
        }

        Ok(())
    }

    fn visit_multi_var_statement(
        &mut self,
        multi_var_statement: &crate::rlox::stmt::MultiVarStatement,
    ) -> Result<(), LoxError> {
        self.convert_statements(&multi_var_statement.vars)?;
        Ok(())
    }

    fn visit_block_statement(
        &mut self,
        block_statement: &crate::rlox::stmt::BlockStatement,
    ) -> Result<(), LoxError> {
        self.begin_scope();
        if let Err(e) = self.convert_statements(&block_statement.statements) {
            self.end_scope();
            return Err(e);
        }
        self.end_scope();
        Ok(())
    }

    fn visit_branch_statement(
        &mut self,
        branch_statement: &crate::rlox::stmt::BranchStatement,
    ) -> Result<(), LoxError> {
        self.convert_expression(&branch_statement.condition)?;
        let jump_false = self.current_chunk().write(OpCode::JumpIfFalse(0), (0, 0));
        self.current_chunk().write(OpCode::Pop, (0, 0));

        self.convert_statement(&branch_statement.then_branch)?;

        if let Some(eb) = &branch_statement.else_branch {
            let jump = self.current_chunk().write(OpCode::Jump(0), (0, 0));
            self.patch_jump_opcode(jump_false);
            self.current_chunk().write(OpCode::Pop, (0, 0));
            self.convert_statement(eb)?;
            self.patch_jump_opcode(jump);
        } else {
            self.patch_jump_opcode(jump_false);
            self.current_chunk().write(OpCode::Pop, (0, 0));
        }

        Ok(())
    }

    fn visit_while_statement(
        &mut self,
        while_statement: &crate::rlox::stmt::WhileStatement,
    ) -> Result<(), LoxError> {
        let loop_start = self.current_chunk().len();
        let pre = self.loop_body_depth;
        self.loop_body_depth = self.scopes.depth;

        self.convert_expression(&while_statement.condition)?;
        let jump_false = self.current_chunk().write(OpCode::JumpIfFalse(0), (0, 0));
        self.current_chunk().write(OpCode::Pop, (0, 0));
        self.convert_statement(&while_statement.body)?;
        self.handle_continue_jump();

        if let Some(incr) = &while_statement.increment {
            self.convert_statement(incr)?;
        }
        let cur = self.current_chunk().len();
        self.function
            .chunk
            .write(OpCode::JumpForward(cur - loop_start + 1), (0, 0));
        self.patch_jump_opcode(jump_false);
        self.current_chunk().write(OpCode::Pop, (0, 0));
        self.handle_break_jump();

        self.loop_body_depth = pre;

        Ok(())
    }

    fn visit_continue_statement(
        &mut self,
        continue_statement: &crate::rlox::stmt::ContinueStatement,
    ) -> Result<(), LoxError> {
        self.scopes
            .will_delete_var_by_depth(self.loop_body_depth)
            .into_iter()
            .for_each(|c| {
                self.current_chunk().write(c, (0, 0));
            });
        self.continue_position.push(
            self.function
                .chunk
                .write(OpCode::Jump(0), continue_statement.token.position),
        );
        Ok(())
    }

    fn visit_break_statement(
        &mut self,
        break_statement: &crate::rlox::stmt::BreakStatement,
    ) -> Result<(), LoxError> {
        self.scopes
            .will_delete_var_by_depth(self.loop_body_depth)
            .into_iter()
            .for_each(|c| {
                self.current_chunk().write(c, (0, 0));
            });
        self.break_position.push(
            self.function
                .chunk
                .write(OpCode::Jump(0), break_statement.token.position),
        );
        Ok(())
    }

    fn visit_function_statement(
        &mut self,
        function_statement: &crate::rlox::stmt::FunctionStatement,
    ) -> Result<(), LoxError> {
        let name = function_statement.name.lexeme.clone();
        let arity = function_statement.params.len();
        let mut convertor = Convertor::new(&name, FuncType::Normal);

        let depth = convertor.scopes.depth;
        for param in &function_statement.params {
            convertor
                .scopes
                .define_variable(param.lexeme.clone(), depth);
        }

        let mut func = convertor.convert(&function_statement.body)?;
        func.arity = arity;

        let func = Rc::new(func);
        self.current_chunk()
            .write(OpCode::Load(func.into()), function_statement.name.position);

        // self.current_chunk()
        //     .write(OpCode::DefineGlobal(name), function_statement.name.position);
        if self.scopes.depth == 0 {
            self.current_chunk()
                .write(OpCode::DefineGlobal(name), function_statement.name.position);
        } else {
            self.scopes.define_variable(name, self.scopes.depth);
        }
        Ok(())
    }

    fn visit_return_statement(
        &mut self,
        return_statement: &crate::rlox::stmt::ReturnStatement,
    ) -> Result<(), LoxError> {
        self.is_returned = true;
        if let Some(value) = &return_statement.value {
            self.convert_expression(value)?;
        } else {
            self.current_chunk().write(
                OpCode::Load(Literal::Nil),
                return_statement.key_word.position,
            );
        }

        self.current_chunk()
            .write(OpCode::Return, return_statement.key_word.position);

        Ok(())
    }

    fn visit_class_statement(
        &mut self,
        class_statement: &crate::rlox::stmt::ClassStatement,
    ) -> Result<(), LoxError> {
        todo!()
    }
}
