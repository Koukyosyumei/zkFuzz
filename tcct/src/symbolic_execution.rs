use crate::parser_user::{
    DebugAccess, DebugAssignOp, DebugExpression, DebugExpressionInfixOpcode,
    DebugExpressionPrefixOpcode, DebugStatement,
};
use num_bigint_dig::BigInt;
use program_structure::ast::Access;
use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;
use program_structure::ast::Statement;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
enum SymbolicValue {
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
            SymbolicValue::BinaryOp(lhs, op, rhs) => write!(f, "({:?} {:?} {:?})", lhs, op, rhs),
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
    constraints: Vec<SymbolicValue>,
}

impl SymbolicState {
    pub fn new() -> Self {
        SymbolicState {
            values: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn set_symval(&mut self, name: String, value: SymbolicValue) {
        self.values.insert(name, value);
    }

    pub fn push_symconstraint(&mut self, constraint: SymbolicValue) {
        self.constraints.push(constraint);
    }

    pub fn get_symval(&self, name: &str) -> Option<&SymbolicValue> {
        self.values.get(name)
    }
}

pub struct SymbolicExecutor {
    pub cur_state: SymbolicState,
    pub final_states: Vec<SymbolicState>,
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        SymbolicExecutor {
            cur_state: SymbolicState::new(),
            final_states: Vec::new(),
        }
    }

    pub fn execute(&mut self, statement: &DebugStatement) {
        match &statement.0 {
            Statement::IfThenElse {
                meta,
                cond,
                if_case,
                else_case,
                ..
            } => {
                let condition = self.evaluate_expression(&DebugExpression(cond.clone()));
                // Create a branch in the symbolic state
                let mut if_state = self.cur_state.clone();
                if_state.push_symconstraint(condition.clone());
                self.execute(&DebugStatement(*if_case.clone()));
                if let Some(else_stmt) = else_case {
                    let mut else_state = self.cur_state.clone();
                    else_state.push_symconstraint(SymbolicValue::UnaryOp(
                        DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                        Box::new(condition),
                    ));
                    self.execute(&DebugStatement(*else_stmt.clone()));
                }
            }
            Statement::While {
                meta, cond, stmt, ..
            } => {
                // Symbolic execution of loops is complex. This is a simplified approach.
                let condition = self.evaluate_expression(&DebugExpression(cond.clone()));
                self.cur_state.push_symconstraint(condition);
                self.execute(&DebugStatement(*stmt.clone()));
                // Note: This doesn't handle loop invariants or fixed-point computation
            }
            Statement::Return { meta, value, .. } => {
                let return_value = self.evaluate_expression(&DebugExpression(value.clone()));
                // Handle return value (e.g., store in a special "return" variable)
                self.cur_state
                    .set_symval("__return__".to_string(), return_value);
            }
            Statement::InitializationBlock {
                meta,
                xtype,
                initializations,
                ..
            } => {
                for init in initializations {
                    self.execute(&DebugStatement(init.clone()));
                }
            }
            Statement::Declaration {
                meta,
                xtype,
                name,
                dimensions,
                is_constant,
                ..
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
            }
            Statement::Substitution {
                meta,
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
                        "{}[{:?}]",
                        var,
                        &access
                            .iter()
                            .map(|arg0: &Access| DebugAccess(arg0.clone()))
                            .collect::<Vec<_>>()
                    )
                };
                self.cur_state.set_symval(var_name, value);
            }
            Statement::MultSubstitution {
                meta, lhe, op, rhe, ..
            } => {
                let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()));
                let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()));
                // Handle multiple substitution (simplified)
                self.cur_state.push_symconstraint(SymbolicValue::BinaryOp(
                    Box::new(lhs),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                    Box::new(rhs),
                ));
            }
            /*
            Statement::UnderscoreSubstitution { op, rhe, .. } => {
                // Underscore substitution doesn't affect the symbolic state
                // But we might want to evaluate the rhe for side effects
                self.evaluate_expression(&DebugExpression(rhe.clone()));
            },
            */
            Statement::ConstraintEquality { meta, lhe, rhe } => {
                let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()));
                let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()));
                self.cur_state.push_symconstraint(SymbolicValue::BinaryOp(
                    Box::new(lhs),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                    Box::new(rhs),
                ));
            }
            /*
            Statement::LogCall { args, .. } => {
                // Logging doesn't affect the symbolic state
                // But we might want to evaluate the args for side effects
                for arg in args {
                    self.evaluate_expression(&DebugExpression(arg.clone()));
                }
            },
            */
            Statement::Block { meta, stmts, .. } => {
                for stmt in stmts {
                    self.execute(&DebugStatement(stmt.clone()));
                }
            }
            Statement::Assert { meta, arg, .. } => {
                let condition = self.evaluate_expression(&DebugExpression(arg.clone()));
                self.cur_state.push_symconstraint(condition);
            }
            // Handle other statement types
            _ => {
                println!("Unhandled statement type: {:?}", statement);
            }
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
                    SymbolicValue::Variable(format!("{}", name))
                    //SymbolicValue::Variable(format!("{}[{:?}]", name, access))
                }
            }
            Expression::InfixOp {
                meta,
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
                meta,
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
                meta,
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
            Expression::ArrayInLine { meta, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone())))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::Tuple { meta, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone())))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::UniformArray {
                meta,
                value,
                dimension,
                ..
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
