use program_structure::abstract_syntax_tree::ast::{AssignOp, Statement};

#[derive(Default)]
pub struct ASTStats {
    num_variables: usize,
    num_statements: usize,
    num_if_then_else: usize,
    num_while: usize,
    num_constraint_equality: usize,
    num_assign_var: usize,
    num_assign_constraint_signal: usize,
    num_assign_signal: usize,
    loc_constraint_equality: usize,
    loc_assign_constraint_signal: usize,
    loc_assign_signal: usize,
}

impl ASTStats {
    pub fn collect_stats(&mut self, statement: &Statement) {
        self.num_statements += 1;

        match statement {
            Statement::Declaration { .. } => {
                self.num_variables += 1;
            }
            Statement::InitializationBlock {
                initializations, ..
            } => {
                for stmt in initializations {
                    self.collect_stats(stmt);
                }
            }
            Statement::ConstraintEquality { .. } => {
                self.num_constraint_equality += 1;
                self.loc_constraint_equality += self.num_statements;
            }
            Statement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.collect_stats(stmt);
                }
            }
            Statement::IfThenElse {
                if_case, else_case, ..
            } => {
                self.num_if_then_else += 1;
                self.collect_stats(if_case);
                if let Some(ec) = else_case {
                    self.collect_stats(ec);
                }
            }
            Statement::While { stmt, .. } => {
                self.num_while += 1;
                self.collect_stats(stmt);
            }
            Statement::Substitution { op, .. } => match op {
                AssignOp::AssignVar => {
                    self.num_assign_var += 1;
                }
                AssignOp::AssignConstraintSignal => {
                    self.num_assign_constraint_signal += 1;
                    self.loc_assign_constraint_signal += self.num_statements;
                }
                AssignOp::AssignSignal => {
                    self.num_assign_signal += 1;
                    self.loc_assign_signal += self.num_statements;
                }
            },
            _ => {} // Handle other statement types as needed.
        }
    }

    pub fn get_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{}",
            self.num_statements,
            self.num_variables,
            self.num_if_then_else,
            self.num_while,
            self.num_constraint_equality,
            self.num_assign_var,
            self.num_assign_constraint_signal,
            self.num_assign_signal,
            if self.num_constraint_equality == 0 {
                0
            } else {
                self.loc_constraint_equality / self.num_constraint_equality
            },
            if self.num_assign_constraint_signal == 0 {
                0
            } else {
                self.loc_assign_constraint_signal / self.num_assign_constraint_signal
            },
            if self.num_assign_signal == 0 {
                0
            } else {
                self.loc_assign_signal / self.num_assign_signal
            },
        )
        .to_string()
    }
}
