use crate::parser_user::{
    DebugAccess, DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode,
    ExtendedStatement,
};
use num_bigint_dig::BigInt;
use program_structure::ast::Access;
use program_structure::ast::AssignOp;
use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;
use program_structure::ast::Statement;
use std::collections::HashMap;
use std::fmt;

pub fn simplify_statement(statement: &Statement) -> Statement {
    match &statement {
        Statement::Substitution {
            meta: _,
            var,
            access,
            op,
            rhe,
        } => {
            // Check if the RHS contains an InlineSwitchOp
            if let Expression::InlineSwitchOp {
                meta,
                cond,
                if_true,
                if_false,
            } = rhe
            {
                let if_stmt = Statement::Substitution {
                    meta: meta.clone(),
                    var: var.clone(),
                    access: access.clone(),
                    op: *op, // Assuming simple assignment
                    rhe: *if_true.clone(),
                };

                let else_stmt = Statement::Substitution {
                    meta: meta.clone(),
                    var: var.clone(),
                    access: access.clone(),
                    op: *op, // Assuming simple assignment
                    rhe: *if_false.clone(),
                };

                Statement::IfThenElse {
                    meta: meta.clone(),
                    cond: *cond.clone(),
                    if_case: Box::new(if_stmt),
                    else_case: Some(Box::new(else_stmt)),
                }
            } else {
                statement.clone() // No InlineSwitchOp, return as-is
            }
        }
        Statement::IfThenElse {
            meta,
            cond,
            if_case,
            else_case,
        } => {
            if else_case.is_none() {
                Statement::IfThenElse {
                    meta: meta.clone(),
                    cond: cond.clone(),
                    if_case: Box::new(simplify_statement(if_case)),
                    else_case: None,
                }
            } else {
                Statement::IfThenElse {
                    meta: meta.clone(),
                    cond: cond.clone(),
                    if_case: Box::new(simplify_statement(if_case)),
                    else_case: Some(Box::new(simplify_statement(&else_case.clone().unwrap()))),
                }
            }
        }
        Statement::Block { meta, stmts } => Statement::Block {
            meta: meta.clone(),
            stmts: stmts
                .iter()
                .map(|arg0: &Statement| simplify_statement(arg0))
                .collect::<Vec<_>>(),
        },
        _ => statement.clone(),
    }
}

#[derive(Clone)]
pub enum SymbolicValue {
    Constant(BigInt),
    Variable(String),
    BinaryOp(
        Box<SymbolicValue>,
        DebugExpressionInfixOpcode,
        Box<SymbolicValue>,
    ),
    Conditional(Box<SymbolicValue>, Box<SymbolicValue>, Box<SymbolicValue>),
    UnaryOp(DebugExpressionPrefixOpcode, Box<SymbolicValue>),
    Array(Vec<SymbolicValue>),
    Tuple(Vec<SymbolicValue>),
    UniformArray(Box<SymbolicValue>, Box<SymbolicValue>),
    Call(String, Vec<SymbolicValue>),
}

impl fmt::Debug for SymbolicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolicValue::Constant(value) => write!(f, "{}", value),
            SymbolicValue::Variable(name) => write!(f, "{}", name),
            SymbolicValue::BinaryOp(lhs, op, rhs) => write!(f, "({:?} {:?} {:?})", op, lhs, rhs),
            SymbolicValue::Conditional(cond, if_branch, else_branch) => {
                write!(f, "({:?} {:?} {:?})", cond, if_branch, else_branch)
            }
            SymbolicValue::UnaryOp(op, expr) => write!(f, "({:?} {:?})", op, expr),
            _ => write!(f, "unknown symbolic value"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolicState {
    values: HashMap<String, SymbolicValue>,
    trace_constraints: Vec<SymbolicValue>,
    side_constraints: Vec<SymbolicValue>,
}

impl SymbolicState {
    pub fn new() -> Self {
        SymbolicState {
            values: HashMap::new(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
        }
    }

    pub fn set_symval(&mut self, name: String, value: SymbolicValue) {
        self.values.insert(name, value);
    }

    pub fn push_trace_constraint(&mut self, constraint: SymbolicValue) {
        self.trace_constraints.push(constraint);
    }

    pub fn push_side_constraint(&mut self, constraint: SymbolicValue) {
        self.side_constraints.push(constraint);
    }

    pub fn get_symval(&self, name: &str) -> Option<&SymbolicValue> {
        self.values.get(name)
    }
}

pub struct SymbolicExecutor {
    pub cur_state: SymbolicState,
    pub stack_states: Vec<SymbolicState>,
    pub block_end_states: Vec<SymbolicState>,
    pub final_states: Vec<SymbolicState>,
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        SymbolicExecutor {
            cur_state: SymbolicState::new(),
            stack_states: vec![SymbolicState::new()],
            block_end_states: Vec::new(),
            final_states: Vec::new(),
        }
    }

    fn execute_next_block(&mut self, statements: &Vec<ExtendedStatement>, cur_bid: usize) {
        let stack_states = self.block_end_states.clone();
        self.block_end_states.clear();
        for state in &stack_states.clone() {
            self.cur_state = state.clone();
            self.execute(statements, cur_bid + 1);
        }
    }

    pub fn execute(&mut self, statements: &Vec<ExtendedStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            match &statements[cur_bid] {
                ExtendedStatement::DebugStatement(stmt) => {
                    match stmt {
                        Statement::InitializationBlock {
                            initializations, ..
                        } => {
                            for init in initializations {
                                self.execute(
                                    &vec![ExtendedStatement::DebugStatement(init.clone())],
                                    0,
                                );
                            }
                            self.block_end_states = vec![self.cur_state.clone()];
                            self.execute_next_block(statements, cur_bid);
                        }
                        Statement::Block { stmts, .. } => {
                            if cur_bid < stmts.len() {
                                //for state in &self.stack_states.clone() {
                                //    self.cur_state = state.clone();
                                self.execute(
                                    &stmts
                                        .iter()
                                        .map(|arg0: &Statement| {
                                            ExtendedStatement::DebugStatement(arg0.clone())
                                        })
                                        .collect::<Vec<_>>(),
                                    0,
                                );
                                //}
                                self.execute_next_block(statements, cur_bid);
                            }
                        }
                        Statement::IfThenElse {
                            cond,
                            if_case,
                            else_case,
                            ..
                        } => {
                            let condition =
                                self.evaluate_expression(&DebugExpression(cond.clone()));
                            // Create a branch in the symbolic state
                            let mut if_state = self.cur_state.clone();
                            let mut else_state = self.cur_state.clone();
                            let tmp_cur_bid = cur_bid;

                            if_state.push_trace_constraint(condition.clone());
                            self.cur_state = if_state.clone();
                            self.execute(
                                &vec![ExtendedStatement::DebugStatement(*if_case.clone())],
                                0,
                            );
                            self.execute_next_block(statements, cur_bid);

                            if let Some(else_stmt) = else_case {
                                else_state.push_trace_constraint(SymbolicValue::UnaryOp(
                                    DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                    Box::new(condition),
                                ));
                                self.cur_state = else_state;
                                self.execute(
                                    &vec![ExtendedStatement::DebugStatement(*else_stmt.clone())],
                                    0,
                                );
                                self.execute_next_block(statements, cur_bid);
                            }
                        }
                        Statement::While { cond, stmt, .. } => {
                            // Symbolic execution of loops is complex. This is a simplified approach.
                            let condition =
                                self.evaluate_expression(&DebugExpression(cond.clone()));
                            self.cur_state.push_trace_constraint(condition);
                            self.execute(
                                &vec![ExtendedStatement::DebugStatement(*stmt.clone())],
                                0,
                            );
                            self.execute_next_block(statements, cur_bid);
                            // Note: This doesn't handle loop invariants or fixed-point computation
                        }
                        Statement::Return { value, .. } => {
                            let return_value =
                                self.evaluate_expression(&DebugExpression(value.clone()));
                            // Handle return value (e.g., store in a special "return" variable)
                            self.cur_state
                                .set_symval("__return__".to_string(), return_value);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Declaration {
                            name, dimensions, ..
                        } => {
                            let var_name = if dimensions.is_empty() {
                                name.clone()
                            } else {
                                //"todo".to_string()
                                format!(
                                    "{}[{:?}]",
                                    name,
                                    &dimensions
                                        .iter()
                                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                                        .collect::<Vec<_>>()
                                )
                            };
                            let value = SymbolicValue::Variable(var_name.clone());
                            self.cur_state.set_symval(var_name, value);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Substitution {
                            meta: _,
                            var,
                            access,
                            op,
                            rhe,
                        } => {
                            let value = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            let var_name = if access.is_empty() {
                                var.clone()
                            } else {
                                //format!("{}", var)
                                format!(
                                    "{}{:?}",
                                    var,
                                    &access
                                        .iter()
                                        .map(|arg0: &Access| DebugAccess(arg0.clone()))
                                        .collect::<Vec<_>>()
                                )
                            };
                            self.cur_state.set_symval(var_name.clone(), value.clone());
                            let cont = SymbolicValue::BinaryOp(
                                Box::new(SymbolicValue::Variable(var_name)),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(value),
                            );
                            self.cur_state.push_trace_constraint(cont.clone());
                            if let AssignOp::AssignConstraintSignal = op {
                                self.cur_state.push_side_constraint(cont.clone());
                            }
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::MultSubstitution { lhe, op, rhe, .. } => {
                            let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()));
                            let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            // Handle multiple substitution (simplified)
                            let cont = SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(rhs),
                            );
                            self.cur_state.push_trace_constraint(cont.clone());
                            if let AssignOp::AssignConstraintSignal = op {
                                self.cur_state.push_side_constraint(cont);
                            }
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::ConstraintEquality { meta: _, lhe, rhe } => {
                            let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()));
                            let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            let cond = SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(rhs),
                            );
                            //self.cur_state.push_trace_constraint(cond.clone());
                            self.cur_state.push_side_constraint(cond);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Assert { arg, .. } => {
                            let condition = self.evaluate_expression(&DebugExpression(arg.clone()));
                            self.cur_state.push_trace_constraint(condition);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::UnderscoreSubstitution { op, rhe, .. } => {
                            // Underscore substitution doesn't affect the symbolic state
                        }
                        Statement::LogCall { args, .. } => {
                            // Logging doesn't affect the symbolic state
                        }
                        // Handle other statement types
                        _ => {
                            println!("Unhandled statement type: {:?}", statements[cur_bid]);
                        }
                    }
                }
                ExtendedStatement::Ret => {
                    self.final_states.push(self.cur_state.clone());
                }
            }
        } else {
            self.block_end_states.push(self.cur_state.clone());
        }
    }

    fn evaluate_expression(&self, expr: &DebugExpression) -> SymbolicValue {
        match &expr.0 {
            Expression::Number(meta, value) => SymbolicValue::Constant(value.clone()),
            Expression::Variable { name, access, meta } => {
                if access.is_empty() {
                    self.cur_state
                        .get_symval(&name)
                        .cloned()
                        .unwrap_or_else(|| SymbolicValue::Variable(name.clone()))
                } else {
                    SymbolicValue::Variable(format!(
                        "{}{:?}",
                        name,
                        &access
                            .iter()
                            .map(|arg0: &Access| DebugAccess(arg0.clone()))
                            .collect::<Vec<_>>()
                    ))
                }
            }
            Expression::InfixOp {
                meta: _,
                lhe,
                infix_op,
                rhe,
            } => {
                let lhs = self.evaluate_expression(&DebugExpression(*lhe.clone()));
                let rhs = self.evaluate_expression(&DebugExpression(*rhe.clone()));
                SymbolicValue::BinaryOp(
                    Box::new(lhs),
                    DebugExpressionInfixOpcode(infix_op.clone()),
                    Box::new(rhs),
                )
            }
            Expression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => {
                let expr = self.evaluate_expression(&DebugExpression(*rhe.clone()));
                SymbolicValue::UnaryOp(
                    DebugExpressionPrefixOpcode(prefix_op.clone()),
                    Box::new(expr),
                )
            }
            Expression::InlineSwitchOp {
                meta: _,
                cond,
                if_true,
                if_false,
            } => {
                let condition = self.evaluate_expression(&DebugExpression(*cond.clone()));
                let true_branch = self.evaluate_expression(&DebugExpression(*if_true.clone()));
                let false_branch = self.evaluate_expression(&DebugExpression(*if_false.clone()));
                SymbolicValue::Conditional(
                    Box::new(condition),
                    Box::new(true_branch),
                    Box::new(false_branch),
                )
            }
            Expression::ParallelOp { rhe, .. } => {
                self.evaluate_expression(&DebugExpression(*rhe.clone()))
            }
            Expression::ArrayInLine { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone())))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::Tuple { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone())))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::UniformArray {
                value, dimension, ..
            } => {
                let evaluated_value = self.evaluate_expression(&DebugExpression(*value.clone()));
                let evaluated_dimension =
                    self.evaluate_expression(&DebugExpression(*dimension.clone()));
                SymbolicValue::UniformArray(
                    Box::new(evaluated_value),
                    Box::new(evaluated_dimension),
                )
            }
            Expression::Call { id, args, .. } => {
                let evaluated_args = args
                    .iter()
                    .map(|arg| self.evaluate_expression(&DebugExpression(arg.clone())))
                    .collect();
                SymbolicValue::Call(id.clone(), evaluated_args)
            }
            /*
            DebugExpression::BusCall { id, args, .. } => {
                let evaluated_args = args.iter()
                    .map(|arg| self.evaluate_expression(&DebugExpression(arg.clone())))
                    .collect();
                SymbolicValue::FunctionCall(format!("Bus_{}", id), evaluated_args)
            }
            DebugExpression::AnonymousComp { id, params, signals, .. } => {
                let evaluated_params = params.iter()
                    .map(|param| self.evaluate_expression(&DebugExpression(param.clone())))
                    .collect();
                let evaluated_signals = signals.iter()
                    .map(|signal| self.evaluate_expression(&DebugExpression(signal.clone())))
                    .collect();
                SymbolicValue::FunctionCall(format!("AnonymousComp_{}", id),
                    [evaluated_params, evaluated_signals].concat())
            }*/
            // Handle other expression types
            _ => {
                println!("Unhandled expression type: {:?}", expr);
                SymbolicValue::Variable(format!("Unhandled({:?})", expr))
            }
        }
    }
}
