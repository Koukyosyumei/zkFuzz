mod parser_user;

use std::collections::HashMap;
use std::fmt;
use parser_user::{DebugAssignOp, DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcodem DebugStatement};

#[derive(Clone, Debug)]
enum SymbolicValue {
    Constant(i64),
    Variable(String),
    BinaryOp(
        Box<SymbolicValue>,
        DebugExpressionInfixOpcode,
        Box<SymbolicValue>,
    ),
    UnaryOp(DebugExpressionPrefixOpcode, Box<SymbolicValue>),
}

impl fmt::Debug for SymbolicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolicValue::Constant(value) => write!(f, "{}", value),
            SymbolicValue::Variable(name) => write!(f, "{}", name),
            SymbolicValue::BinaryOp(lhs, op, rhs) => write!(f, "({:?} {:?} {:?})", lhs, op, rhs),
            SymbolicValue::UnaryOp(op, expr) => write!(f, "({:?} {:?})", op, expr),
        }
    }
}

struct SymbolicState {
    values: HashMap<String, SymbolicValue>,
}

impl SymbolicState {
    fn new() -> Self {
        SymbolicState {
            values: HashMap::new(),
        }
    }

    fn set(&mut self, name: String, value: SymbolicValue) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<&SymbolicValue> {
        self.values.get(name)
    }
}

#[derive(Clone, Debug)]
struct SymConstraint {
    lhs: SymbolicValue,
    rhs: SymbolicValue,
}

struct SymConstraintCollector {
    constraints: Vec<SymConstraint>,
}

impl SymConstraintCollector {
    fn new() -> Self {
        SymConstraintCollector {
            constraints: Vec::new(),
        }
    }

    fn add(&mut self, constraint: SymConstraint) {
        self.constraints.push(constraint);
    }
}

struct SymbolicExecutor {
    state: SymbolicState,
    constraints: SymConstraintCollector,
}

impl SymbolicExecutor {
    fn new() -> Self {
        SymbolicExecutor {
            state: SymbolicState::new(),
            constraints: SymConstraintCollector::new(),
        }
    }

    fn execute(&mut self, statement: &DebugStatement) {
        match statement {
            DebugStatement::Substitution {
                var,
                access,
                op,
                rhe,
            } => {
                let value = self.evaluate_expression(rhe);
                let var_name = if access.is_empty() {
                    var.clone()
                } else {
                    format!("{}[{:?}]", var, access)
                };
                self.state.set(var_name, value);
            }
            DebugStatement::SymConstraintEquality { lhe, rhe } => {
                let lhs = self.evaluate_expression(lhe);
                let rhs = self.evaluate_expression(rhe);
                let constraint = SymConstraint { lhs, rhs };
                self.constraints.add(constraint);
            }
            DebugStatement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.execute(stmt);
                }
            }
            // Handle other statement types
            _ => {
                println!("Unhandled statement type: {:?}", statement);
            }
        }
    }

    fn evaluate_expression(&self, expr: &DebugExpression) -> SymbolicValue {
        match expr {
            DebugExpression::Number(_, value) => SymbolicValue::Constant(*value),
            DebugExpression::Variable { name, access } => {
                if access.is_empty() {
                    self.state
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| SymbolicValue::Variable(name.clone()))
                } else {
                    SymbolicValue::Variable(format!("{}[{:?}]", name, access))
                }
            }
            DebugExpression::InfixOp { lhe, infix_op, rhe } => {
                let lhs = self.evaluate_expression(lhe);
                let rhs = self.evaluate_expression(rhe);
                SymbolicValue::BinaryOp(Box::new(lhs), infix_op.clone(), Box::new(rhs))
            }
            DebugExpression::PrefixOp { prefix_op, rhe } => {
                let expr = self.evaluate_expression(rhe);
                SymbolicValue::UnaryOp(prefix_op.clone(), Box::new(expr))
            }
            // Handle other expression types
            _ => {
                println!("Unhandled expression type: {:?}", expr);
                SymbolicValue::Variable(format!("Unhandled({:?})", expr))
            }
        }
    }
}
