use std::{collections::HashMap, rc::Rc};

use super::opcode::OpCode;

#[derive(Default, Debug)]
pub struct Scopes {
    pub variables: Vec<(Rc<String>, usize)>,
    pub var_map: HashMap<Rc<String>, Vec<usize>>,
    pub count: usize,
    pub depth: usize,
}

impl Scopes {
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub fn end_scope(&mut self) -> Vec<OpCode> {
        self.depth -= 1;
        let mut codes = vec![];
        while !self.variables.is_empty() && self.variables.last().unwrap().1 > self.depth {
            codes.push(OpCode::Pop);
            let name = self.variables.pop().unwrap().0;
            self.var_map.entry(name).and_modify(|v| {
                v.pop();
            });
        }
        codes
    }

    pub fn will_delete_var_by_depth(&mut self, depth: usize) -> Vec<OpCode> {
        let mut codes = vec![];
        let mut len = self.variables.iter().filter(|(_, d)| depth < *d).count();
        while len > 0 {
            codes.push(OpCode::Pop);
            len -= 1;
        }

        codes
    }

    pub fn define_variable(&mut self, name: Rc<String>, depth: usize) -> Result<(), ()> {
        if !self.is_variable_at_same_depth(&name, depth) {
            let index = self.variables.len();
            self.variables.push((name.clone(), depth));
            self.var_map
                .entry(name)
                .and_modify(|v| v.push(index))
                .or_insert_with(|| vec![index]);
            return Ok(());
        }
        Err(())
    }

    fn is_variable_at_same_depth(&self, name: &Rc<String>, depth: usize) -> bool {
        if let Some(index) = self.var_map.get(name) {
            if !index.is_empty() {
                for i in index.iter().rev() {
                    if self.variables[*i].1 == depth {
                        return true;
                    }
                }
            }
            return false;
        }
        false
    }

    pub fn find_variable(&self, name: Rc<String>) -> Result<usize, ()> {
        match self.var_map.get(&name) {
            Some(index) => {
                for i in index.iter().rev() {
                    let i = *i;
                    let value = &self.variables[i];
                    if value.0 == name {
                        return Ok(i);
                    }
                }
                Err(())
            }
            None => Err(()),
        }

        // for (i, var) in self.variables.iter().rev().enumerate() {
        //     println!("{}", i);
        //     if name == var.0 {
        //         return Ok(i);
        //     }
        // }

        // Err(())
    }
}
