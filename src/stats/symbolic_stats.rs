use std::collections::{HashMap, HashSet};

use crate::executor::symbolic_value::{SymbolicName, SymbolicValue};

const RESET: &str = "\x1b[0m";
const WHITE: &str = "\x1b[37m";
const BBLACK: &str = "\x1b[90m";

/// Collects statistics about constraints encountered during symbolic execution.
#[derive(Default)]
pub struct ConstraintStatistics {
    pub total_constraints: usize,
    pub constraint_depths: Vec<usize>,
    pub operator_counts: HashMap<String, usize>,
    pub variable_counts: HashMap<SymbolicName, usize>,
    pub constant_counts: usize,
    pub conditional_counts: usize,
    pub array_counts: usize,
    pub function_call_counts: HashMap<usize, usize>,
    pub cache: HashSet<SymbolicValue>,
}

impl ConstraintStatistics {
    /// Creates a new instance of `ConstraintStatistics` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates statistics based on a given symbolic value and its depth in the expression tree.
    ///
    /// # Arguments
    ///
    /// * `value` - The symbolic value to analyze.
    /// * `depth` - The depth level of this value in its expression tree.
    fn update_from_symbolic_value(&mut self, value: &SymbolicValue, depth: usize) {
        match value {
            SymbolicValue::NOP => {}
            SymbolicValue::ConstantInt(_) => {
                self.constant_counts += 1;
            }
            SymbolicValue::ConstantBool(_) => {
                self.constant_counts += 1;
            }
            SymbolicValue::Variable(sym_name) => {
                *self.variable_counts.entry(sym_name.clone()).or_insert(0) += 1;
            }
            SymbolicValue::Assign(lhs, rhs, _, zero_div_info) => {
                *self
                    .operator_counts
                    .entry("Assign".to_string())
                    .or_insert(0) += 1;
                if zero_div_info.is_some() {
                    *self
                        .operator_counts
                        .entry("QuadZeroDiv".to_string())
                        .or_insert(0) += 1;
                }
                self.update_from_symbolic_value(lhs, depth + 1);
                self.update_from_symbolic_value(rhs, depth + 1);
            }
            SymbolicValue::AssignEq(lhs, rhs) | SymbolicValue::AssignTemplParam(lhs, rhs) => {
                *self
                    .operator_counts
                    .entry("AssignEq".to_string())
                    .or_insert(0) += 1;
                self.update_from_symbolic_value(lhs, depth + 1);
                self.update_from_symbolic_value(rhs, depth + 1);
            }
            SymbolicValue::AssignCall(lhs, rhs, _) => {
                *self
                    .operator_counts
                    .entry("AssignCall".to_string())
                    .or_insert(0) += 1;
                self.update_from_symbolic_value(lhs, depth + 1);
                self.update_from_symbolic_value(rhs, depth + 1);
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) => {
                let op_name = format!("{:?}", op);
                *self.operator_counts.entry(op_name).or_insert(0) += 1;
                self.update_from_symbolic_value(lhs, depth + 1);
                self.update_from_symbolic_value(rhs, depth + 1);
            }
            SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
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
            SymbolicValue::UniformArray(value, size) => {
                self.array_counts += 1;
                self.update_from_symbolic_value(value, depth + 1);
                self.update_from_symbolic_value(size, depth + 1);
            }
            SymbolicValue::Call(id, args) => {
                *self.function_call_counts.entry(id.clone()).or_insert(0) += 1;
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

    /// Updates overall statistics with a new constraint.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint to add
    pub fn update(&mut self, constraint: &SymbolicValue) {
        if !self.cache.contains(constraint) {
            self.total_constraints += 1;
            self.cache.insert(constraint.clone());
            self.update_from_symbolic_value(constraint, 0);
        }
    }
}

pub fn print_constraint_summary_statistics_pretty(stats: &ConstraintStatistics) {
    println!(" ┌─────────────────────┬─────────────┐");
    println!(" │ Constraint Type     │     Count   │");
    println!(" ├─────────────────────┼─────────────┤");
    println!(" │ Total               │ {:11} │", stats.total_constraints);
    println!(" │ Constant            │ {:11} │", stats.constant_counts);
    println!(" │ Conditional         │ {:11} │", stats.conditional_counts);
    println!(" │ Array               │ {:11} │", stats.array_counts);
    println!(" └─────────────────────┴─────────────┘");

    let avg_depth = if !stats.constraint_depths.is_empty() {
        stats.constraint_depths.iter().sum::<usize>() as f64 / stats.constraint_depths.len() as f64
    } else {
        0.0
    };
    println!("\n📊 Constraint Depth Statistics:");
    println!(" • Average Depth: {:.2}", avg_depth);
    println!(
        " • Maximum Depth: {}",
        stats.constraint_depths.iter().max().unwrap_or(&0)
    );

    println!("\n🔢 Assign Counts:");
    for op in &["Assign", "AssignEq", "AssignCall", "QuadZeroDiv"] {
        let c = stats.operator_counts.get(*op).unwrap_or(&0);
        println!(
            " • {:<13}: {}{}{}",
            op,
            if *c != 0 { WHITE } else { BBLACK },
            c,
            RESET
        );
    }

    println!("\n🔢 Operator Counts:");
    for op in &[
        "Mul", "Div", "Add", "Sub", "Pow", "IntDiv", "Mod", "ShL", "ShR", "LEq", "GEq", "Lt", "Gt",
        "Eq", "NEq", "BoolOr", "BoolAnd", "BitOr", "BitAnd", "BitXor",
    ] {
        let c = stats.operator_counts.get(*op).unwrap_or(&0);
        println!(
            " • {:<8}: {}{}{}",
            op,
            if *c != 0 { WHITE } else { BBLACK },
            c,
            RESET
        );
    }

    println!("\n📈 Variable Statistics:");
    let var_counts: Vec<usize> = stats.variable_counts.values().cloned().collect();
    let var_avg = if !var_counts.is_empty() {
        var_counts.iter().sum::<usize>() as f64 / var_counts.len() as f64
    } else {
        0.0
    };
    println!(" • Total Number of Variables: {}", var_counts.len());
    println!(" • Average Number of Usage  : {:.2}", var_avg);
    println!(
        " • Maximum Number of Usage  : {}",
        var_counts.iter().max().unwrap_or(&0)
    );

    println!("\n📞 Function Call Statistics:");
    let func_counts: Vec<usize> = stats.function_call_counts.values().cloned().collect();
    let func_avg = if !func_counts.is_empty() {
        func_counts.iter().sum::<usize>() as f64 / func_counts.len() as f64
    } else {
        0.0
    };
    println!(" • Average Count: {:.2}", func_avg);
    println!(
        " • Maximum Count: {}",
        func_counts.iter().max().unwrap_or(&0)
    );
}

pub fn print_constraint_summary_statistics_csv(constraint_stats: &ConstraintStatistics) {
    let mut values = Vec::new();
    values.push(constraint_stats.total_constraints.to_string());
    values.push(constraint_stats.constant_counts.to_string());
    values.push(constraint_stats.conditional_counts.to_string());
    values.push(constraint_stats.array_counts.to_string());

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
        "Assign",
        "AssignEq",
        "AssignCall",
        "QuadZeroDiv",
        "Mul",
        "Div",
        "Add",
        "Sub",
        "Pow",
        "IntDiv",
        "Mod",
        "ShL",
        "ShR",
        "LEq",
        "GEq",
        "Lt",
        "Gt",
        "Eq",
        "NEq",
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
    let num_vars = var_counts.len();
    let var_avg = if !var_counts.is_empty() {
        var_counts.iter().sum::<usize>() as f64 / var_counts.len() as f64
    } else {
        0.0
    };
    values.push(num_vars.to_string());
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
