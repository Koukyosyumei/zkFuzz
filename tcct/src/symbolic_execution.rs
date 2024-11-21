use crate::parser_user::{
    DebugAccess, DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode,
    ExtendedStatement,
};
use colored::Colorize;
use log::trace;
use num_bigint_dig::BigInt;
use program_structure::ast::Access;
use program_structure::ast::AssignOp;
use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;
use program_structure::ast::Statement;
use std::cmp::max;
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
                let mut meta_if = meta.clone();
                meta_if.elem_id = std::usize::MAX - meta.elem_id * 2;
                let if_stmt = Statement::Substitution {
                    meta: meta_if.clone(),
                    var: var.clone(),
                    access: access.clone(),
                    op: *op, // Assuming simple assignment
                    rhe: *if_true.clone(),
                };

                let mut meta_else = meta.clone();
                meta_else.elem_id = std::usize::MAX - (meta.elem_id * 2 + 1);
                let else_stmt = Statement::Substitution {
                    meta: meta_else.clone(),
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
            SymbolicValue::Variable(name) => write!(f, "{}", name.yellow()),
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Eq => {
                    write!(f, "({} {:?} {:?})", format!("{:?}", op).green(), lhs, rhs)
                }
                ExpressionInfixOpcode::NotEq => {
                    write!(f, "({} {:?} {:?})", format!("{:?}", op).green(), lhs, rhs)
                }
                _ => write!(f, "({} {:?} {:?})", format!("{:?}", op), lhs, rhs),
            },
            SymbolicValue::Conditional(cond, if_branch, else_branch) => {
                write!(f, "({:?} {:?} {:?})", cond, if_branch, else_branch)
            }
            SymbolicValue::UnaryOp(op, expr) => match &op.0 {
                ExpressionPrefixOpcode::BoolNot => {
                    write!(f, "({} {:?})", format!("{:?}", op).red(), expr)
                }
                _ => write!(f, "({} {:?})", format!("{:?}", op), expr),
            },
            _ => write!(f, "unknown symbolic value"),
        }
    }
}

#[derive(Clone)]
pub struct SymbolicState {
    values: HashMap<String, SymbolicValue>,
    trace_constraints: Vec<SymbolicValue>,
    side_constraints: Vec<SymbolicValue>,
    depth: usize,
}

impl fmt::Debug for SymbolicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SymbolicState [")?;
        writeln!(f, "  values:")?;
        for (k, v) in self.values.clone().into_iter() {
            writeln!(
                f,
                "    {}: {}",
                k.replace("\n", "").replace("  ", " "),
                format!("{:?}", v.clone())
                    .replace("\n", "")
                    .replace("  ", " ")
            )?;
        }
        writeln!(
            f,
            "  trace_constraints: {}",
            format!("{:?}", self.trace_constraints)
                .replace("\n", "")
                .replace("  ", " ")
                .replace("  ", " ")
        )?;
        writeln!(
            f,
            "  sidec_constraints: {}",
            format!("{:?}", self.side_constraints)
                .replace("\n", "")
                .replace("  ", " ")
                .replace("  ", " ")
        )?;
        writeln!(f, "  depth: {:?}", self.depth)?;
        write!(f, "]")
    }
}

impl SymbolicState {
    pub fn new() -> Self {
        SymbolicState {
            values: HashMap::new(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
            depth: 0_usize,
        }
    }

    pub fn set_symval(&mut self, name: String, value: SymbolicValue) {
        self.values.insert(name, value);
    }

    pub fn get_symval(&self, name: &str) -> Option<&SymbolicValue> {
        self.values.get(name)
    }

    pub fn push_trace_constraint(&mut self, constraint: SymbolicValue) {
        self.trace_constraints.push(constraint);
    }

    pub fn push_side_constraint(&mut self, constraint: SymbolicValue) {
        self.side_constraints.push(constraint);
    }

    pub fn set_depth(&mut self, d: usize) {
        self.depth = d;
    }
    pub fn get_depth(&self) -> usize {
        self.depth
    }
}

#[derive(Default, Debug)]
pub struct ConstraintStatistics {
    total_constraints: usize,
    constraint_depths: Vec<usize>,
    operator_counts: HashMap<String, usize>,
    variable_counts: HashMap<String, usize>,
    constant_counts: usize,
    conditional_counts: usize,
    array_counts: usize,
    tuple_counts: usize,
    function_call_counts: HashMap<String, usize>,
}

impl ConstraintStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_from_symbolic_value(&mut self, value: &SymbolicValue, depth: usize) {
        match value {
            SymbolicValue::Constant(_) => {
                self.constant_counts += 1;
            }
            SymbolicValue::Variable(name) => {
                *self.variable_counts.entry(name.clone()).or_insert(0) += 1;
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) => {
                let op_name = format!("{:?}", op);
                *self.operator_counts.entry(op_name).or_insert(0) += 1;
                self.update_from_symbolic_value(lhs, depth + 1);
                self.update_from_symbolic_value(rhs, depth + 1);
            }
            SymbolicValue::Conditional(cond, if_true, if_false) => {
                self.conditional_counts += 1;
                self.update_from_symbolic_value(cond, depth + 1);
                self.update_from_symbolic_value(if_true, depth + 1);
                self.update_from_symbolic_value(if_false, depth + 1);
            }
            SymbolicValue::UnaryOp(op, expr) => {
                let op_name = format!("{:?}", op);
                *self.operator_counts.entry(op_name).or_insert(0) += 1;
                self.update_from_symbolic_value(expr, depth + 1);
            }
            SymbolicValue::Array(elements) => {
                self.array_counts += 1;
                for elem in elements {
                    self.update_from_symbolic_value(elem, depth + 1);
                }
            }
            SymbolicValue::Tuple(elements) => {
                self.tuple_counts += 1;
                for elem in elements {
                    self.update_from_symbolic_value(elem, depth + 1);
                }
            }
            SymbolicValue::UniformArray(value, size) => {
                self.array_counts += 1;
                self.update_from_symbolic_value(value, depth + 1);
                self.update_from_symbolic_value(size, depth + 1);
            }
            SymbolicValue::Call(name, args) => {
                *self.function_call_counts.entry(name.clone()).or_insert(0) += 1;
                for arg in args {
                    self.update_from_symbolic_value(arg, depth + 1);
                }
            }
        }

        if self.constraint_depths.len() <= depth {
            self.constraint_depths.push(1);
        } else {
            self.constraint_depths[depth] += 1;
        }
    }

    pub fn update(&mut self, constraint: &SymbolicValue) {
        self.total_constraints += 1;
        self.update_from_symbolic_value(constraint, 0);
    }
}

/*
pub fn print_constraint_statistics(constraint_stats: &ConstraintStatistics) {
    println!("Constraint Statistics:");
    println!("Total constraints: {}", constraint_stats.total_constraints);
    println!(
        "Constraint depths: {:?}",
        constraint_stats.constraint_depths
    );
    println!("Operator counts: {:?}", constraint_stats.operator_counts);
    println!("Variable counts: {:?}", constraint_stats.variable_counts);
    println!("Constant counts: {}", constraint_stats.constant_counts);
    println!(
        "Conditional counts: {}",
        constraint_stats.conditional_counts
    );
    println!("Array counts: {}", constraint_stats.array_counts);
    println!("Tuple counts: {}", constraint_stats.tuple_counts);
    println!(
        "Function call counts: {:?}",
        constraint_stats.function_call_counts
    );
}*/

pub fn print_constraint_summary_statistics(constraint_stats: &ConstraintStatistics) {
    let headers = vec![
        "Total_Constraints",
        "Constant_Counts",
        "Conditional_Counts",
        "Array_Counts",
        "Tuple_Counts",
        "Avg_Depth",
        "Max_Depth",
        "Count_Mul",
        "Count_Div",
        "Count_Add",
        "Count_Sub",
        "Count_Pow",
        "Count_IntDiv",
        "Count_Mod",
        "Count_ShiftL",
        "Count_ShiftR",
        "Count_LesserEq",
        "Count_GreaterEq",
        "Count_Lesser",
        "Count_Greater",
        "Count_Eq",
        "Count_NotEq",
        "Count_BoolOr",
        "Count_BoolAnd",
        "Count_BitOr",
        "Count_BitAnd",
        "Count_BitXor",
        "Variable_Avg_Count",
        "Variable_Max_Count",
        "Function_Avg_Count",
        "Function_Max_Count",
    ];
    println!("{}", headers.join(","));

    let mut values = Vec::new();
    values.push(constraint_stats.total_constraints.to_string());
    values.push(constraint_stats.constant_counts.to_string());
    values.push(constraint_stats.conditional_counts.to_string());
    values.push(constraint_stats.array_counts.to_string());
    values.push(constraint_stats.tuple_counts.to_string());

    let avg_depth = if !constraint_stats.constraint_depths.is_empty() {
        constraint_stats.constraint_depths.iter().sum::<usize>() as f64
            / constraint_stats.constraint_depths.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", avg_depth));
    values.push(
        constraint_stats
            .constraint_depths
            .iter()
            .max()
            .unwrap_or(&0)
            .to_string(),
    );

    for op in &[
        "Mul",
        "Div",
        "Add",
        "Sub",
        "Pow",
        "IntDiv",
        "Mod",
        "ShiftL",
        "ShiftR",
        "LesserEq",
        "GreaterEq",
        "Lesser",
        "Greater",
        "Eq",
        "NotEq",
        "BoolOr",
        "BoolAnd",
        "BitOr",
        "BitAnd",
        "BitXor",
    ] {
        values.push(
            constraint_stats
                .operator_counts
                .get(*op)
                .unwrap_or(&0)
                .to_string(),
        );
    }

    let var_counts: Vec<usize> = constraint_stats.variable_counts.values().cloned().collect();
    let var_avg = if !var_counts.is_empty() {
        var_counts.iter().sum::<usize>() as f64 / var_counts.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", var_avg));
    values.push(var_counts.iter().max().unwrap_or(&0).to_string());

    let func_counts: Vec<usize> = constraint_stats
        .function_call_counts
        .values()
        .cloned()
        .collect();
    let func_avg = if !func_counts.is_empty() {
        func_counts.iter().sum::<usize>() as f64 / func_counts.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", func_avg));
    values.push(func_counts.iter().max().unwrap_or(&0).to_string());

    println!("{}", values.join(","));
}

pub struct SymbolicExecutor {
    pub cur_state: SymbolicState,
    pub block_end_states: Vec<SymbolicState>,
    pub final_states: Vec<SymbolicState>,
    // constraints
    pub trace_constraint_stats: ConstraintStatistics,
    pub side_constraint_stats: ConstraintStatistics,
    // useful stats
    pub max_depth: usize,
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        SymbolicExecutor {
            cur_state: SymbolicState::new(),
            block_end_states: Vec::new(),
            final_states: Vec::new(),
            trace_constraint_stats: ConstraintStatistics::new(),
            side_constraint_stats: ConstraintStatistics::new(),
            max_depth: 0,
        }
    }

    fn expand_all_stack_states(
        &mut self,
        statements: &Vec<ExtendedStatement>,
        cur_bid: usize,
        depth: usize,
    ) {
        let stack_states = self.block_end_states.clone();
        self.block_end_states.clear();
        for state in &stack_states.clone() {
            self.cur_state = state.clone();
            self.cur_state.set_depth(depth);
            self.execute(statements, cur_bid);
        }
    }

    pub fn execute(&mut self, statements: &Vec<ExtendedStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.max_depth = max(self.max_depth, self.cur_state.get_depth());
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
                            self.expand_all_stack_states(
                                statements,
                                cur_bid + 1,
                                self.cur_state.get_depth(),
                            );
                        }
                        Statement::Block { meta, stmts, .. } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            if cur_bid < stmts.len() {
                                self.execute(
                                    &stmts
                                        .iter()
                                        .map(|arg0: &Statement| {
                                            ExtendedStatement::DebugStatement(arg0.clone())
                                        })
                                        .collect::<Vec<_>>(),
                                    0,
                                );
                                self.expand_all_stack_states(
                                    statements,
                                    cur_bid + 1,
                                    self.cur_state.get_depth(),
                                );
                            }
                        }
                        Statement::IfThenElse {
                            meta,
                            cond,
                            if_case,
                            else_case,
                            ..
                        } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let condition =
                                self.evaluate_expression(&DebugExpression(cond.clone()), true);
                            self.trace_constraint_stats.update(&condition);

                            // Save the current state
                            let cur_depth = self.cur_state.get_depth();
                            let stack_states = self.block_end_states.clone();

                            // Create a branch in the symbolic state
                            let mut if_state = self.cur_state.clone();
                            let mut else_state = self.cur_state.clone();

                            if_state.push_trace_constraint(condition.clone());
                            if_state.set_depth(cur_depth + 1);
                            self.cur_state = if_state.clone();
                            self.execute(
                                &vec![ExtendedStatement::DebugStatement(*if_case.clone())],
                                0,
                            );
                            self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);

                            if let Some(else_stmt) = else_case {
                                let mut stack_states_if_true = self.block_end_states.clone();
                                self.block_end_states = stack_states;
                                else_state.push_trace_constraint(SymbolicValue::UnaryOp(
                                    DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                    Box::new(condition),
                                ));
                                else_state.set_depth(cur_depth + 1);
                                self.cur_state = else_state;
                                self.execute(
                                    &vec![ExtendedStatement::DebugStatement(*else_stmt.clone())],
                                    0,
                                );
                                self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                                self.block_end_states.append(&mut stack_states_if_true);
                            }
                        }
                        Statement::While {
                            meta, cond, stmt, ..
                        } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            // Symbolic execution of loops is complex. This is a simplified approach.
                            let condition =
                                self.evaluate_expression(&DebugExpression(cond.clone()), true);
                            self.trace_constraint_stats.update(&condition);

                            self.cur_state.push_trace_constraint(condition);
                            self.execute(
                                &vec![ExtendedStatement::DebugStatement(*stmt.clone())],
                                0,
                            );
                            self.expand_all_stack_states(
                                statements,
                                cur_bid + 1,
                                self.cur_state.get_depth(),
                            );
                            // Note: This doesn't handle loop invariants or fixed-point computation
                        }
                        Statement::Return { meta, value, .. } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let return_value =
                                self.evaluate_expression(&DebugExpression(value.clone()), true);
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
                            meta,
                            var,
                            access,
                            op,
                            rhe,
                        } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let original_value =
                                self.evaluate_expression(&DebugExpression(rhe.clone()), false);
                            let value =
                                self.evaluate_expression(&DebugExpression(rhe.clone()), true);

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
                                Box::new(SymbolicValue::Variable(var_name.clone())),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(value),
                            );
                            self.cur_state.push_trace_constraint(cont.clone());
                            self.trace_constraint_stats.update(&cont);

                            if let AssignOp::AssignConstraintSignal = op {
                                let original_cont = SymbolicValue::BinaryOp(
                                    Box::new(SymbolicValue::Variable(var_name.clone())),
                                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                    Box::new(original_value),
                                );
                                self.cur_state.push_side_constraint(original_cont.clone());
                                self.side_constraint_stats.update(&original_cont);
                            }
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::MultSubstitution {
                            meta, lhe, op, rhe, ..
                        } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let simple_lhs =
                                self.evaluate_expression(&DebugExpression(lhe.clone()), false);
                            let simple_rhs =
                                self.evaluate_expression(&DebugExpression(rhe.clone()), false);
                            let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()), true);
                            let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()), true);

                            // Handle multiple substitution (simplified)
                            let cont = SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(rhs),
                            );
                            self.cur_state.push_trace_constraint(cont.clone());
                            self.trace_constraint_stats.update(&cont);
                            if let AssignOp::AssignConstraintSignal = op {
                                // Handle multiple substitution (simplified)
                                let simple_cont = SymbolicValue::BinaryOp(
                                    Box::new(simple_lhs),
                                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                    Box::new(simple_rhs),
                                );
                                self.cur_state.push_side_constraint(simple_cont.clone());
                                self.side_constraint_stats.update(&simple_cont);
                            }
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::ConstraintEquality { meta, lhe, rhe } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let original_lhs =
                                self.evaluate_expression(&DebugExpression(lhe.clone()), false);
                            let original_rhs =
                                self.evaluate_expression(&DebugExpression(rhe.clone()), false);
                            let lhs = self.evaluate_expression(&DebugExpression(lhe.clone()), true);
                            let rhs = self.evaluate_expression(&DebugExpression(rhe.clone()), true);

                            let original_cond = SymbolicValue::BinaryOp(
                                Box::new(original_lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(original_rhs),
                            );
                            let cond = SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(rhs),
                            );

                            self.cur_state.push_trace_constraint(cond.clone());
                            self.trace_constraint_stats.update(&cond);
                            self.cur_state.push_side_constraint(original_cond.clone());
                            self.side_constraint_stats.update(&original_cond);

                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Assert { meta, arg, .. } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            let condition =
                                self.evaluate_expression(&DebugExpression(arg.clone()), true);
                            self.cur_state.push_trace_constraint(condition.clone());
                            self.trace_constraint_stats.update(&condition);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::UnderscoreSubstitution {
                            meta,
                            op: _,
                            rhe: _,
                            ..
                        } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            // Underscore substitution doesn't affect the symbolic state
                        }
                        Statement::LogCall { meta, args: _, .. } => {
                            trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            // Logging doesn't affect the symbolic state
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

    fn evaluate_expression(&self, expr: &DebugExpression, substiture_var: bool) -> SymbolicValue {
        match &expr.0 {
            Expression::Number(_meta, value) => SymbolicValue::Constant(value.clone()),
            Expression::Variable {
                name,
                access,
                meta: _,
            } => {
                if access.is_empty() {
                    if substiture_var {
                        self.cur_state
                            .get_symval(&name)
                            .cloned()
                            .unwrap_or_else(|| SymbolicValue::Variable(name.clone()))
                    } else {
                        SymbolicValue::Variable(name.clone())
                    }
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
                let lhs = self.evaluate_expression(&DebugExpression(*lhe.clone()), substiture_var);
                let rhs = self.evaluate_expression(&DebugExpression(*rhe.clone()), substiture_var);
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
                let expr = self.evaluate_expression(&DebugExpression(*rhe.clone()), substiture_var);
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
                let condition =
                    self.evaluate_expression(&DebugExpression(*cond.clone()), substiture_var);
                let true_branch =
                    self.evaluate_expression(&DebugExpression(*if_true.clone()), substiture_var);
                let false_branch =
                    self.evaluate_expression(&DebugExpression(*if_false.clone()), substiture_var);
                SymbolicValue::Conditional(
                    Box::new(condition),
                    Box::new(true_branch),
                    Box::new(false_branch),
                )
            }
            Expression::ParallelOp { rhe, .. } => {
                self.evaluate_expression(&DebugExpression(*rhe.clone()), substiture_var)
            }
            Expression::ArrayInLine { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone()), substiture_var))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::Tuple { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| self.evaluate_expression(&DebugExpression(v.clone()), substiture_var))
                    .collect();
                SymbolicValue::Array(elements)
            }
            Expression::UniformArray {
                value, dimension, ..
            } => {
                let evaluated_value =
                    self.evaluate_expression(&DebugExpression(*value.clone()), substiture_var);
                let evaluated_dimension =
                    self.evaluate_expression(&DebugExpression(*dimension.clone()), substiture_var);
                SymbolicValue::UniformArray(
                    Box::new(evaluated_value),
                    Box::new(evaluated_dimension),
                )
            }
            Expression::Call { id, args, .. } => {
                let evaluated_args = args
                    .iter()
                    .map(|arg| {
                        self.evaluate_expression(&DebugExpression(arg.clone()), substiture_var)
                    })
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
