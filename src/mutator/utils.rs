use core::panic;
use std::fmt;
use std::rc::Rc;

use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::ToPrimitive;
use num_traits::{One, Signed, Zero};
use rustc_hash::{FxHashMap, FxHashSet};

use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;
use serde_json::{json, Value};

use crate::executor::debug_ast::DebuggableExpressionInfixOpcode;
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_setting::SymbolicExecutorSetting;
use crate::executor::symbolic_value::{
    evaluate_binary_op, evaluate_binary_op_integer_mode, extract_variables_from_symbolic_value,
    normalize_to_bool, normalize_to_int, val_for_relational_operators, OwnerName, QuadraticPoly,
    SymbolicAccess, SymbolicLibrary, SymbolicName, SymbolicValue, SymbolicValueRef,
};

#[derive(Clone)]
pub enum UnderConstrainedType {
    UnusedOutput,
    UnexpectedInput(usize, String),
    NonDeterministic(SymbolicName, String, BigInt),
}

/// Represents the result of a constraint verification process.
#[derive(Clone)]
pub enum VerificationResult {
    UnderConstrained(UnderConstrainedType),
    OverConstrained,
    WellConstrained,
}

impl fmt::Display for VerificationResult {
    /// Formats the `VerificationResult` for display, using color-coded output.
    ///
    /// # Returns
    /// A `fmt::Result` indicating success or failure of the formatting
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            VerificationResult::UnderConstrained(typ) => match typ {
                UnderConstrainedType::UnusedOutput => {
                    "👻 UnderConstrained (Unused-Output) 👻".red().bold().to_string()
                }
                UnderConstrainedType::UnexpectedInput(_pos, violated_condition) => {
                    format!("{} {}", "🧟 UnderConstrained (Unexpected-Input) 🧟\n║           Violated Condition:".red().bold(), violated_condition)
                }
                UnderConstrainedType::NonDeterministic(_sym_name, name, value) => format!(
                    "🔥 UnderConstrained (Non-Deterministic) 🔥\n║           ➡️ `{}` is expected to be `{}`",
                    name, value
                )
                .red()
                .bold().to_string(),
            },
            VerificationResult::OverConstrained => "💣 OverConstrained 💣".yellow().bold().to_string(),
            VerificationResult::WellConstrained => "✅ WellConstrained ✅".green().bold().to_string(),
        };
        write!(f, "{output}")
    }
}

impl VerificationResult {
    pub fn to_json(&self) -> Value {
        match self {
            VerificationResult::UnderConstrained(typ) => match typ {
                UnderConstrainedType::UnusedOutput => {
                    json!({"1_type": "UnderConstrained-UnusedOutput"})
                }
                UnderConstrainedType::UnexpectedInput(pos, _violated_condition) => {
                    json!({"1_type": "UnderConstrained-UnexpectedInput", "2_violated_condition":json!({"pos":pos})})
                }
                UnderConstrainedType::NonDeterministic(_sym_name, name, value) => {
                    json!({"1_type": "UnderConstrained-NonDeterministic", "2_expected_output": json!({"name": name, "value":value.to_string()})})
                }
            },
            VerificationResult::OverConstrained => json!({"1_type": "OverConstrained"}),
            VerificationResult::WellConstrained => json!({"1_type": "WellConstrained"}),
        }
    }
}

/// Represents a counterexample when constraints are found to be invalid.
#[derive(Clone)]
pub struct CounterExample {
    pub flag: VerificationResult,
    pub target_output: Option<SymbolicName>,
    pub assignment: FxHashMap<SymbolicName, BigInt>,
}

impl CounterExample {
    pub fn to_json_with_meta(
        &self,
        lookup: &FxHashMap<usize, String>,
        meta: &FxHashMap<String, String>,
    ) -> Value {
        let mut base_json = json!({
            "5_flag": self.flag.to_json(),
        });

        for (key, value) in meta {
            base_json[key] = json!(value);
        }

        if let Some(target) = &self.target_output {
            base_json["6_target_output"] = json!(target.lookup_fmt(lookup));
        }

        base_json["7_assignment"] = json!(self
            .assignment
            .iter()
            .map(|(var_name, value)| (var_name.lookup_fmt(lookup), value.to_string()))
            .collect::<FxHashMap<String, String>>());

        base_json
    }

    /// Generates a detailed, user-friendly debug output for the counterexample.
    ///
    /// # Parameters
    /// - `lookup`: A hash map associating variable IDs with their string representations.
    ///
    /// # Returns
    /// A formatted string containing the counterexample details.
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        let mut s = "".to_string();
        s += &format!(
            "{}",
            "╔══════════════════════════════════════════════════════════════╗\n".red()
        );
        s += &format!("{}", "║".red());
        s += &format!(
            "🚨 {}                                           ",
            "Counter Example:".on_bright_red().white().bold()
        );
        s += &format!("{}", "║\n".red());
        s += &format!("{}", "║".red());
        s += &format!("    {} \n", self.flag);
        s += &format!("{}", "║".red());
        s += &format!("    {} \n", "🔍 Assignment Details:".blue().bold());

        let mut is_target_output = false;
        for (var_name, value) in &self.assignment {
            if var_name.owner.len() == 1 {
                s += &format!("{}", "║".red());
                if let Some(to) = &self.target_output {
                    if *to == *var_name {
                        is_target_output = true;
                        s += &format!(
                            "           {} {}{}{} \n",
                            "➡️".cyan(),
                            var_name.lookup_fmt(lookup).on_magenta().white().bold(),
                            " = ".on_magenta().white().bold(),
                            value.to_string().on_magenta().bright_yellow().bold()
                        );
                    }
                }
                if !is_target_output {
                    s += &format!(
                        "           {} {} = {} \n",
                        "➡️".cyan(),
                        var_name.lookup_fmt(lookup).magenta().bold(),
                        value.to_string().bright_yellow()
                    );
                } else {
                    is_target_output = false;
                }
            }
        }
        for (var_name, value) in &self.assignment {
            if var_name.owner.len() != 1 {
                s += &format!("{}", "║".red());
                s += &format!(
                    "           {} {} = {} \n",
                    "➡️".cyan(),
                    var_name.lookup_fmt(lookup).magenta().bold(),
                    value.to_string().bright_yellow()
                );
            }
        }
        s += &format!(
            "{}",
            "╚══════════════════════════════════════════════════════════════╝\n".red()
        );

        s
    }
}

/// Determines if a given verification result indicates a vulnerability.
///
/// # Parameters
/// - `vr`: The `VerificationResult` to evaluate.
///
/// # Returns
/// `true` if the result indicates a vulnerability, `false` otherwise.
pub fn is_vulnerable(vr: &VerificationResult) -> bool {
    match vr {
        VerificationResult::UnderConstrained(_) => true,
        VerificationResult::OverConstrained => true,
        VerificationResult::WellConstrained => false,
    }
}

/// Configures the settings for the verification process.
pub struct BaseVerificationConfig {
    pub target_template_name: String,
    pub prime: BigInt,
    pub range: BigInt,
    pub quick_mode: bool,
    pub heuristics_mode: bool,
    pub progress_interval: usize,
    pub template_param_names: Vec<String>,
    pub template_param_values: Vec<Expression>,
}

/// Determines whether a collection of symbolic values contains a binary equality check against zero.  
///
/// This function scans through a list of symbolic values, searching for binary patterns (`x * (1 - x) == 0`).  
/// If such a pattern is found and is part of a recognizable binary check structure, the function returns `true`.
///
/// # Parameters
/// - `sym_vals`: A slice of `SymbolicValueRef` representing symbolic expressions to analyze.
/// - `max_level`: The maximum depth allowed for recognizing binary patterns.
///
/// # Returns
/// - `true` if a binary equality check (`x * (1 - x) == 0`) is detected within the allowed depth.
/// - `false` otherwise.
///
/// # Internal Behavior
/// - The function identifies binary equality checks by inspecting `SymbolicValue::BinaryOp` expressions.
/// - If one of the operands is a zero constant, it invokes `is_binary_check` to verify whether the expression  
///   follows a structured binary multiplication pattern.
/// - The helper function `is_binary_check` ensures that a multiplication pattern exists (`x * y`) and delegates  
///   further pattern matching to `matches_binary_pattern`, which checks for the form `(x - 1) * x == 0`.
pub fn is_containing_binary_check(sym_vals: &[SymbolicValueRef], max_level: usize) -> bool {
    for sv in sym_vals {
        if let SymbolicValue::BinaryOp(
            lhs,
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            rhs,
        ) = sv.as_ref()
        {
            if let SymbolicValue::ConstantInt(rv) = rhs.as_ref() {
                if rv.is_zero() && is_binary_check(lhs, max_level) {
                    return true;
                }
            } else if let SymbolicValue::ConstantInt(lv) = lhs.as_ref() {
                if lv.is_zero() && is_binary_check(rhs, max_level) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_binary_check(expr: &SymbolicValueRef, max_level: usize) -> bool {
    if let SymbolicValue::BinaryOp(
        sub_lhs,
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
        sub_rhs,
    ) = expr.as_ref()
    {
        return matches_binary_pattern(sub_lhs, sub_rhs, max_level)
            || matches_binary_pattern(sub_rhs, sub_lhs, max_level);
    }
    false
}

fn matches_binary_pattern(
    var: &SymbolicValueRef,
    expr: &SymbolicValueRef,
    max_level: usize,
) -> bool {
    if let SymbolicValue::Variable(left_name) = var.as_ref() {
        if left_name.owner.len() <= max_level {
            if let SymbolicValue::BinaryOp(
                sub_lhs,
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
                sub_rhs,
            ) = expr.as_ref()
            {
                match (sub_lhs.as_ref(), sub_rhs.as_ref()) {
                    (SymbolicValue::Variable(right_name), SymbolicValue::ConstantInt(rv))
                    | (SymbolicValue::ConstantInt(rv), SymbolicValue::Variable(right_name)) => {
                        if left_name == right_name && rv.is_one() {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    false
}

/// Evaluates a set of constraints given a variable assignment.
///
/// # Parameters
/// - `prime`: The prime modulus for computations.
/// - `constraints`: A slice of symbolic values representing the constraints to evaluate.
/// - `assignment`: A hash map of variable assignments.
///
/// # Returns
/// `true` if all constraints are satisfied, `false` otherwise.
pub fn evaluate_constraints(
    prime: &BigInt,
    constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> bool {
    if constraints.is_empty() {
        true
    } else {
        constraints.iter().all(|constraint| {
            let sv = evaluate_symbolic_value(prime, constraint, assignment, symbolic_library);
            match sv {
                Some(SymbolicValue::ConstantBool(b)) => b,
                Some(v) => {
                    panic!(
                        "Non-bool output value is detected when evaluating a constraint: {}",
                        v.lookup_fmt(&symbolic_library.id2name)
                    )
                }
                _ => {
                    panic!("Non-bool output value is detected when evaluating a constraint: None",)
                }
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

pub fn gather_potential_zero_division(
    trace: &[SymbolicValueRef],
) -> Vec<(usize, (Vec<QuadraticPoly>, Vec<QuadraticPoly>))> {
    let mut result = Vec::new();
    for (i, inst) in trace.iter().enumerate() {
        match inst.as_ref() {
            SymbolicValue::Assign(_, _, _, Some((num, div))) => {
                result.push((i, (num.clone(), div.clone())));
            }
            _ => {}
        }
    }
    result
}

/// Gathers runtime mutable inputs from a symbolic execution trace.
///
/// This function analyzes a symbolic execution trace to identify inputs that are mutable during runtime.
/// It inspects symbolic values to determine whether specific input variables are directly or indirectly
/// influenced by runtime computations. The function returns a mapping of instruction indices to their
/// associated mutation directions (`Left` or `Right`), indicating how the inputs are used in binary
/// operations.
///
/// # Parameters
/// - `trace`: A slice of symbolic values representing the execution trace to be analyzed. Each symbolic
///   value corresponds to a step in the trace.
/// - `symbolic_library`: A mutable reference to the symbolic library, which provides metadata (e.g., mappings
///   for symbolic names) used for interpreting the trace.
/// - `input_variables`: A set of symbolic names corresponding to the input variables that should be monitored
///   for runtime mutability.
///
/// # Returns
/// A map where:
/// - The key is the index of an instruction in the `trace`.
/// - The value is a `Direction` indicating whether the mutable input is on the left-hand side (`Left`) or
///   right-hand side (`Right`) of a binary operation.
pub fn gather_runtime_mutable_inputs(
    trace: &[SymbolicValueRef],
    symbolic_library: &mut SymbolicLibrary,
    input_variables: &FxHashSet<SymbolicName>,
) -> FxHashMap<usize, Direction> {
    let mut used_variables = FxHashSet::default();
    let mut result = FxHashMap::default();
    for (i, inst) in trace.iter().enumerate() {
        match inst.as_ref() {
            SymbolicValue::ConstantBool(..) => {}
            SymbolicValue::Assign(lhs, rhs, _, _)
            | SymbolicValue::AssignEq(lhs, rhs)
            | SymbolicValue::AssignCall(lhs, rhs, _) => {
                if let SymbolicValue::Variable(_sym_name) = lhs.as_ref() {
                    extract_variables_from_symbolic_value(&rhs, &mut used_variables);
                } else {
                    panic!(
                        "Left hand of the assignment is not a variable: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    );
                }
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) | SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
                if let ExpressionInfixOpcode::Eq = op.0 {
                    if let SymbolicValue::Variable(var_name) = &**lhs {
                        if input_variables.contains(var_name) {
                            extract_variables_from_symbolic_value(&rhs, &mut used_variables);
                            if !used_variables.contains(var_name) {
                                result.insert(i, Direction::Left);
                            }
                        }
                    } else if let SymbolicValue::Variable(var_name) = &**rhs {
                        if input_variables.contains(var_name) {
                            extract_variables_from_symbolic_value(&lhs, &mut used_variables);
                            if !used_variables.contains(var_name) {
                                result.insert(i, Direction::Right);
                            }
                        }
                    }
                }

                extract_variables_from_symbolic_value(&lhs, &mut used_variables);
                extract_variables_from_symbolic_value(&rhs, &mut used_variables);
            }
            SymbolicValue::UnaryOp(_op, expr) => {
                extract_variables_from_symbolic_value(&expr, &mut used_variables);
            }
            _ => {}
        }
    }
    result
}

/// Simulates the execution of a symbolic trace, evaluating the values of symbolic variables.
///
/// This function processes a symbolic trace step by step, updating the provided `assignment` with the values
/// of symbolic variables as determined by the trace's operations. It checks whether each operation is valid
/// based on the current assignment and symbolic library, and if any error occurs (e.g., unsatisfied condition),
/// it marks the simulation as unsuccessful and returns the index of the failure.
///
/// # Parameters
/// - `prime`: A reference to the prime modulus used for modular arithmetic.
/// - `trace`: A slice of references to symbolic values representing the symbolic trace to be simulated.
/// - `runtime_mutable_positions`: A map of runtime mutable positions.
/// - `assignment`: A mutable hash map of symbolic variable names to their corresponding `BigInt` values.
/// - `symbolic_library`: A mutable reference to a symbolic library containing the definitions of symbolic values.
///
/// # Returns
/// A tuple:
/// - `bool`: Indicates whether the symbolic trace simulation was successful (`true`) or failed (`false`).
/// - `usize`: The index of the trace operation that caused the failure, if any; otherwise, `0`.
///
/// # Behavior
/// 1. The function iterates over each symbolic value in the trace:
///    - For constants, it checks whether they match the expected value.
///    - For assignments, it evaluates the right-hand side expression and updates the `assignment` map.
///    - For binary and unary operations, it checks the result of the operation and compares it to the expected value.
/// 2. If an operation fails (e.g., an unsatisfied condition), the simulation is marked as unsuccessful,
///    and the index of the failing operation is returned.
///
/// # Errors
/// - If the left-hand side of an assignment is not a variable, the function will panic.
/// - If there is an unassigned variable in the expression being evaluated, the function will panic.
pub fn emulate_symbolic_trace(
    prime: &BigInt,
    trace: &[SymbolicValueRef],
    runtime_mutable_positions: &FxHashMap<usize, Direction>,
    assignment: &mut FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> Option<(bool, usize)> {
    let mut success = true;
    let mut failure_pos = 0;
    // let input_variables: FxHashSet<SymbolicName> = assignment.keys().cloned().collect();
    for (i, inst) in trace.iter().enumerate() {
        // println!("{}", inst.lookup_fmt(&symbolic_library.id2name));
        match inst.as_ref() {
            SymbolicValue::NOP => {}
            SymbolicValue::ConstantBool(b) => {
                if !b {
                    success = false;
                    failure_pos = i;
                }
            }
            SymbolicValue::Assign(lhs, rhs, _, _)
            | SymbolicValue::AssignEq(lhs, rhs)
            | SymbolicValue::AssignTemplParam(lhs, rhs)
            | SymbolicValue::AssignCall(lhs, rhs, _) => {
                if let SymbolicValue::Variable(sym_name) = lhs.as_ref() {
                    let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
                    match &rhs_val {
                        Some(SymbolicValue::NOP) => {
                            if !assignment.contains_key(sym_name) {
                                assignment.insert(sym_name.clone(), BigInt::zero());
                            }
                        }
                        Some(SymbolicValue::ConstantInt(num)) => {
                            assignment.insert(sym_name.clone(), num.clone());
                        }
                        Some(SymbolicValue::ConstantBool(b)) => {
                            assignment.insert(
                                sym_name.clone(),
                                if *b { BigInt::one() } else { BigInt::zero() },
                            );
                        }
                        Some(SymbolicValue::Array(arr)) => {
                            for (i, a) in arr.iter().enumerate() {
                                if let SymbolicValue::ConstantInt(v) = a.as_ref() {
                                    let mut name = sym_name.clone();
                                    let mut accsess = if name.access.is_some() {
                                        name.access.unwrap().clone()
                                    } else {
                                        Vec::new()
                                    };
                                    accsess.push(SymbolicAccess::ArrayAccess(
                                        SymbolicValue::ConstantInt(BigInt::from(i)),
                                    ));
                                    name.access = Some(accsess);
                                    name.update_hash();
                                    assignment.insert(name, v.clone());
                                } else {
                                    todo!("Support nested-arrays for template parameters");
                                }
                            }
                        }
                        None => {
                            return None;
                        }
                        _ => {
                            success = false;
                            failure_pos = i;
                        }
                    }
                } else {
                    panic!(
                        "Left hand of the assignment is not a variable: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    );
                }
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) => {
                let mut lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
                let mut rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);

                if let Some(dir) = runtime_mutable_positions.get(&i) {
                    match dir {
                        Direction::Left => {
                            if let SymbolicValue::Variable(var_name) = &**lhs {
                                if let Some(SymbolicValue::ConstantInt(ref num)) = rhs_val {
                                    assignment.insert(var_name.clone(), num.clone());
                                    lhs_val = rhs_val.clone();
                                }
                            }
                        }
                        Direction::Right => {
                            if let SymbolicValue::Variable(var_name) = &**rhs {
                                if let Some(SymbolicValue::ConstantInt(ref num)) = lhs_val {
                                    assignment.insert(var_name.clone(), num.clone());
                                    rhs_val = lhs_val.clone();
                                }
                            }
                        }
                    }
                }

                let (normalized_lhs, normalized_rhs) = match &op.0 {
                    // Convert booleans to integers for arithmetic or bitwise operators
                    ExpressionInfixOpcode::Add
                    | ExpressionInfixOpcode::Sub
                    | ExpressionInfixOpcode::Mul
                    | ExpressionInfixOpcode::Pow
                    | ExpressionInfixOpcode::Div
                    | ExpressionInfixOpcode::IntDiv
                    | ExpressionInfixOpcode::Mod
                    | ExpressionInfixOpcode::BitOr
                    | ExpressionInfixOpcode::BitAnd
                    | ExpressionInfixOpcode::BitXor
                    | ExpressionInfixOpcode::ShiftL
                    | ExpressionInfixOpcode::ShiftR
                    | ExpressionInfixOpcode::Lesser
                    | ExpressionInfixOpcode::Greater
                    | ExpressionInfixOpcode::LesserEq
                    | ExpressionInfixOpcode::GreaterEq
                    | ExpressionInfixOpcode::Eq
                    | ExpressionInfixOpcode::NotEq => (
                        normalize_to_int(&lhs_val.unwrap(), prime),
                        normalize_to_int(&rhs_val.unwrap(), prime),
                    ),
                    // Keep booleans as they are for logical operators
                    ExpressionInfixOpcode::BoolAnd | ExpressionInfixOpcode::BoolOr => (
                        normalize_to_bool(&lhs_val.unwrap(), prime),
                        normalize_to_bool(&rhs_val.unwrap(), prime),
                    ), //_ => (lhs.clone(), rhs.clone()), // Default case
                };

                let flag = match (&normalized_lhs, &normalized_rhs) {
                    (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                        match op.0 {
                            ExpressionInfixOpcode::Lesser => {
                                val_for_relational_operators(&(lv % prime), prime)
                                    < val_for_relational_operators(&(rv % prime), prime)
                            }
                            ExpressionInfixOpcode::Greater => {
                                val_for_relational_operators(&(lv % prime), prime)
                                    > val_for_relational_operators(&(rv % prime), prime)
                            }
                            ExpressionInfixOpcode::LesserEq => {
                                val_for_relational_operators(&(lv % prime), prime)
                                    <= val_for_relational_operators(&(rv % prime), prime)
                            }
                            ExpressionInfixOpcode::GreaterEq => {
                                val_for_relational_operators(&(lv % prime), prime)
                                    >= val_for_relational_operators(&(rv % prime), prime)
                            }
                            ExpressionInfixOpcode::Eq => lv % prime == rv % prime,
                            ExpressionInfixOpcode::NotEq => lv % prime != rv % prime,
                            _ => panic!(
                                "Non-Boolean Operation: {}",
                                inst.lookup_fmt(&symbolic_library.id2name)
                            ),
                        }
                    }
                    (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => {
                        match &op.0 {
                            ExpressionInfixOpcode::BoolAnd => *lv && *rv,
                            ExpressionInfixOpcode::BoolOr => *lv || *rv,
                            _ => todo!(),
                        }
                    }
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    ),
                };
                if !flag {
                    success = false;
                    failure_pos = i;
                }
            }
            SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
                let mut lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
                let mut rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);

                if let Some(dir) = runtime_mutable_positions.get(&i) {
                    match dir {
                        Direction::Left => {
                            if let SymbolicValue::Variable(var_name) = &**lhs {
                                if let Some(SymbolicValue::ConstantInt(ref num)) = rhs_val {
                                    assignment.insert(var_name.clone(), num.clone());
                                    lhs_val = rhs_val.clone();
                                }
                            }
                        }
                        Direction::Right => {
                            if let SymbolicValue::Variable(var_name) = &**rhs {
                                if let Some(SymbolicValue::ConstantInt(ref num)) = lhs_val {
                                    assignment.insert(var_name.clone(), num.clone());
                                    rhs_val = lhs_val.clone();
                                }
                            }
                        }
                    }
                }

                let (normalized_lhs, normalized_rhs) = match &op.0 {
                    // Convert booleans to integers for arithmetic or bitwise operators
                    ExpressionInfixOpcode::Add
                    | ExpressionInfixOpcode::Sub
                    | ExpressionInfixOpcode::Mul
                    | ExpressionInfixOpcode::Pow
                    | ExpressionInfixOpcode::Div
                    | ExpressionInfixOpcode::IntDiv
                    | ExpressionInfixOpcode::Mod
                    | ExpressionInfixOpcode::BitOr
                    | ExpressionInfixOpcode::BitAnd
                    | ExpressionInfixOpcode::BitXor
                    | ExpressionInfixOpcode::ShiftL
                    | ExpressionInfixOpcode::ShiftR
                    | ExpressionInfixOpcode::Lesser
                    | ExpressionInfixOpcode::Greater
                    | ExpressionInfixOpcode::LesserEq
                    | ExpressionInfixOpcode::GreaterEq
                    | ExpressionInfixOpcode::Eq
                    | ExpressionInfixOpcode::NotEq => (
                        normalize_to_int(&lhs_val.unwrap(), prime),
                        normalize_to_int(&rhs_val.unwrap(), prime),
                    ),
                    // Keep booleans as they are for logical operators
                    ExpressionInfixOpcode::BoolAnd | ExpressionInfixOpcode::BoolOr => (
                        normalize_to_bool(&lhs_val.unwrap(), prime),
                        normalize_to_bool(&rhs_val.unwrap(), prime),
                    ), //_ => (lhs.clone(), rhs.clone()), // Default case
                };

                let flag = match (&normalized_lhs, &normalized_rhs) {
                    (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                        match op.0 {
                            ExpressionInfixOpcode::Lesser => lv % prime < rv % prime,
                            ExpressionInfixOpcode::Greater => lv % prime > rv % prime,
                            ExpressionInfixOpcode::LesserEq => lv % prime <= rv % prime,
                            ExpressionInfixOpcode::GreaterEq => lv % prime >= rv % prime,
                            ExpressionInfixOpcode::Eq => lv % prime == rv % prime,
                            ExpressionInfixOpcode::NotEq => lv % prime != rv % prime,
                            _ => panic!(
                                "Non-Boolean Operation: {}",
                                inst.lookup_fmt(&symbolic_library.id2name)
                            ),
                        }
                    }
                    (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => {
                        match &op.0 {
                            ExpressionInfixOpcode::BoolAnd => *lv && *rv,
                            ExpressionInfixOpcode::BoolOr => *lv || *rv,
                            _ => todo!(),
                        }
                    }
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    ),
                };
                if !flag {
                    success = false;
                    failure_pos = i;
                }
            }
            SymbolicValue::UnaryOp(op, expr) => {
                let expr_val = evaluate_symbolic_value(prime, expr, assignment, symbolic_library);
                let flag = match &expr_val {
                    Some(SymbolicValue::ConstantBool(rv)) => match op.0 {
                        ExpressionPrefixOpcode::BoolNot => !rv,
                        _ => panic!(
                            "Unassigned variables exist: {}",
                            inst.lookup_fmt(&symbolic_library.id2name)
                        ),
                    },
                    _ => panic!(
                        "Non-Boolean Operation: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    ),
                };
                if !flag {
                    success = false;
                    failure_pos = i;
                }
            }
            _ => {
                let val = evaluate_symbolic_value(prime, inst, assignment, symbolic_library);
                match &val.unwrap() {
                    SymbolicValue::ConstantBool(b) => {
                        if !b {
                            success = false;
                            failure_pos = i;
                        }
                    }
                    SymbolicValue::ConstantInt(v) => {
                        if v.is_zero() {
                            success = false;
                            failure_pos = i;
                        }
                    }
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        inst.lookup_fmt(&symbolic_library.id2name)
                    ),
                }
            }
        }
    }

    Some((success, failure_pos))
}

/// Evaluates a symbolic value within the given context of a symbolic library and variable assignments.
///
/// This function recursively evaluates a symbolic value, resolving constants, variables, arrays,
/// and expressions to their concrete values where possible. The evaluation respects the modular
/// arithmetic defined by the given prime modulus and handles symbolic expressions such as
/// binary and unary operations, conditionals, and function calls.
///
/// # Parameters
/// - `prime`: A reference to the prime modulus used for modular arithmetic.
/// - `value`: A reference to the symbolic value to evaluate. This can be a constant, variable, or a more complex expression.
/// - `assignment`: A hash map containing the current assignment of symbolic variables to their resolved `BigInt` values.
/// - `symbolic_library`: A mutable reference to the symbolic library that contains metadata about symbolic values and functions.
///
/// # Returns
/// - A `SymbolicValue` representing the evaluated result. This could be a resolved constant, a partially evaluated symbolic structure,
///   or the result of a function call or expression.
///
/// # Behavior
/// 1. **Constant Evaluation**: If the value is a constant, it is returned directly.
/// 2. **Variable Resolution**: If the value is a variable, the function retrieves its value from the `assignment` map. If the variable
///    is not found, the function panics.
/// 3. **Array Evaluation**: Arrays and uniform arrays are evaluated element-wise.
/// 4. **Expression Evaluation**:
///    - **Assignments**: Evaluates the left-hand side and right-hand side of the assignment, checking their equality.
///    - **Binary Operations**: Evaluates the operands and applies the specified operator.
///    - **Unary Operations**: Evaluates the operand and applies the specified unary operator.
/// 5. **Conditionals**: Evaluates the condition and returns the result of the appropriate branch (then or else).
/// 6. **Function Calls**: Executes the function body with the provided arguments, returning the result of the function.
///
/// # Errors
/// - Panics if:
///   - A variable referenced in the value is not present in the `assignment` map.
///   - An unsupported operation or invalid symbolic structure is encountered.
///   - A required operation cannot be applied due to type mismatches (e.g., non-integer operations on integers).
///
/// # Notes
/// - The function assumes all symbolic expressions and structures are well-formed.
/// - Modular arithmetic is applied where applicable, with values reduced by the given prime.
/// - The function supports partial evaluation of symbolic expressions when full resolution is not possible.
pub fn evaluate_symbolic_value(
    prime: &BigInt,
    value: &SymbolicValue,
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> Option<SymbolicValue> {
    match value {
        SymbolicValue::NOP => Some(SymbolicValue::NOP),
        SymbolicValue::ConstantBool(_b) => Some(value.clone()),
        SymbolicValue::ConstantInt(_v) => Some(value.clone()),
        SymbolicValue::Variable(sym_name) => {
            if !assignment.contains_key(sym_name) {
                None
            } else {
                Some(SymbolicValue::ConstantInt(
                    assignment.get(sym_name).unwrap().clone(),
                ))
            }
        }
        SymbolicValue::Array(elements) => Some(SymbolicValue::Array(
            elements
                .iter()
                .map(|e| {
                    Rc::new(
                        evaluate_symbolic_value(prime, e, assignment, symbolic_library).unwrap(),
                    )
                })
                .collect(),
        )),
        SymbolicValue::UniformArray(elem, counts) => {
            let evaled_elem = evaluate_symbolic_value(prime, elem, assignment, symbolic_library);
            let evaled_counts =
                evaluate_symbolic_value(prime, counts, assignment, symbolic_library);
            if evaled_elem.is_none() || evaled_counts.is_none() {
                return None;
            }

            if let Some(SymbolicValue::ConstantInt(c)) = evaled_counts {
                Some(SymbolicValue::Array(vec![
                    Rc::new(evaled_elem.unwrap());
                    c.to_usize().unwrap()
                ]))
            } else {
                Some(SymbolicValue::UniformArray(
                    Rc::new(evaled_elem.unwrap()),
                    Rc::new(evaled_counts.unwrap()),
                ))
            }
        }
        SymbolicValue::Assign(lhs, rhs, _, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            if lhs_val.is_none() || rhs_val.is_none() {
                return None;
            }

            match (&lhs_val.unwrap(), &rhs_val.unwrap()) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                    Some(SymbolicValue::ConstantBool(lv % prime == rv % prime))
                }
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantBool(rv)) => {
                    let rv_to_int = if *rv { BigInt::one() } else { BigInt::zero() };
                    Some(SymbolicValue::ConstantBool(lv % prime == rv_to_int))
                }
                _ => panic!(
                    "Unassigned variables exist: {}",
                    value.lookup_fmt(&symbolic_library.id2name)
                ),
            }
        }
        SymbolicValue::AssignTemplParam(_, _) => Some(SymbolicValue::ConstantBool(true)),
        SymbolicValue::BinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            if lhs_val.is_none() || rhs_val.is_none() {
                return None;
            }

            Some(evaluate_binary_op(
                &lhs_val.unwrap(),
                &rhs_val.unwrap(),
                &prime,
                &op,
            ))
        }
        SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            if lhs_val.is_none() || rhs_val.is_none() {
                return None;
            }

            Some(evaluate_binary_op_integer_mode(
                &lhs_val.unwrap(),
                &rhs_val.unwrap(),
                &prime,
                &op,
            ))
        }
        SymbolicValue::UnaryOp(op, expr) => {
            let expr_val = evaluate_symbolic_value(prime, expr, assignment, symbolic_library);
            if expr_val.is_none() {
                return None;
            }

            match &expr_val.unwrap() {
                SymbolicValue::ConstantInt(rv) => match op.0 {
                    ExpressionPrefixOpcode::Sub => Some(SymbolicValue::ConstantInt(-1 * rv)),
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        value.lookup_fmt(&symbolic_library.id2name)
                    ),
                },
                SymbolicValue::ConstantBool(rv) => match op.0 {
                    ExpressionPrefixOpcode::BoolNot => Some(SymbolicValue::ConstantBool(!rv)),
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        value.lookup_fmt(&symbolic_library.id2name)
                    ),
                },
                _ => todo!("{:?}", value),
            }
        }
        SymbolicValue::Conditional(cond, then_branch, else_branch) => {
            let cond_val = evaluate_symbolic_value(prime, cond, assignment, symbolic_library);
            let then_val =
                evaluate_symbolic_value(prime, then_branch, assignment, symbolic_library);
            let else_val =
                evaluate_symbolic_value(prime, else_branch, assignment, symbolic_library);
            if cond_val.is_none() {
                return None;
            }

            match &cond_val.as_ref().unwrap() {
                SymbolicValue::ConstantBool(true) => then_val,
                SymbolicValue::ConstantBool(false) => else_val,
                SymbolicValue::ConstantInt(num) => {
                    if num.is_positive() {
                        then_val
                    } else {
                        else_val
                    }
                }
                _ => panic!(
                    "Unassigned variables exist: {}",
                    cond_val.unwrap().lookup_fmt(&symbolic_library.id2name)
                ),
            }
        }
        SymbolicValue::Call(id, args) => {
            let setting = SymbolicExecutorSetting {
                prime: prime.clone(),
                is_input_overwrite_disabled: false,
                only_initialization_blocks: false,
                off_trace: true,
                keep_track_constraints: false,
                substitute_output: false,
                propagate_assignments: true,
                constraint_assert_dissabled: false,
            };
            let mut subse = SymbolicExecutor::new(symbolic_library, &setting);

            let func = subse.symbolic_library.function_library[id].clone();
            for i in 0..(func.function_argument_names.len()) {
                let sym_name = SymbolicName::new(
                    func.function_argument_names[i],
                    subse.cur_state.owner_name.clone(),
                    None,
                );
                let evaled_arg =
                    evaluate_symbolic_value(prime, &args[i], assignment, subse.symbolic_library);
                if evaled_arg.is_none() {
                    return None;
                }
                subse
                    .cur_state
                    .set_rc_sym_val(sym_name, Rc::new(evaled_arg.unwrap()));
            }
            subse.execute(&func.body.clone(), 0);
            if subse.execution_failed {
                None
            } else {
                let return_name =
                    SymbolicName::new(usize::MAX, subse.cur_state.owner_name.clone(), None);
                if !subse
                    .cur_state
                    .symbol_binding_map
                    .contains_key(&return_name)
                {
                    return None;
                }
                let return_value =
                    (*subse.cur_state.symbol_binding_map[&return_name].clone()).clone();
                if let SymbolicValue::ConstantInt(_) = &return_value {
                    Some(return_value)
                } else {
                    None
                }
            }
        }
    }
}

/// Evaluates the error of a symbolic value for a given assignment under modular arithmetic.
///
/// This function computes the "error" of a symbolic value when evaluated with a specific assignment
/// of symbolic names to integer values. The error is defined as the absolute difference between
/// expected and actual values modulo a prime number.
///
/// # Parameters
/// - `prime`: The prime modulus used for modular arithmetic.
/// - `value`: The symbolic value to evaluate the error for.
/// - `assignment`: A mapping of symbolic names to their concrete integer values.
/// - `symbolic_library`: A mutable reference to the symbolic library providing variable lookup and
///   other symbolic operations.
///
/// # Returns
/// The computed error as a `BigInt`. A zero error indicates the symbolic value matches the
/// assignment modulo the prime.
///
/// # Panics
/// - If unassigned variables are encountered in the symbolic value.
/// - If unsupported operators are used, such as non-comparison or non-boolean operators.
pub fn evaluate_error_of_symbolic_value(
    prime: &BigInt,
    value: &SymbolicValue,
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> BigInt {
    match value {
        SymbolicValue::NOP => BigInt::zero(),
        SymbolicValue::ConstantBool(b) => {
            if *b {
                BigInt::zero()
            } else {
                BigInt::one()
            }
        }
        SymbolicValue::Assign(lhs, rhs, _, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val.unwrap(), &rhs_val.unwrap()) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                    (lv % prime - rv % prime).abs()
                }
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantBool(flag)) => {
                    (lv % prime - if *flag { BigInt::one() } else { BigInt::zero() }).abs()
                }
                (SymbolicValue::ConstantBool(flag), SymbolicValue::ConstantInt(rv)) => {
                    (rv % prime - if *flag { BigInt::one() } else { BigInt::zero() }).abs()
                }
                (SymbolicValue::ConstantBool(lflag), SymbolicValue::ConstantBool(rflag)) => {
                    if *lflag == *rflag {
                        BigInt::zero()
                    } else {
                        BigInt::one()
                    }
                }
                _ => panic!("Unassigned variables exist"),
            }
        }
        SymbolicValue::AssignTemplParam(_, _) => BigInt::zero(),
        SymbolicValue::BinaryOp(lhs, op, rhs) | SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (lhs_val.as_ref().unwrap(), rhs_val.as_ref().unwrap()) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match &op.0 {
                    ExpressionInfixOpcode::Lesser => lv % prime + BigInt::one() - rv % prime,
                    ExpressionInfixOpcode::Greater => rv % prime + BigInt::one() - lv % prime,
                    ExpressionInfixOpcode::LesserEq => lv % prime - rv % prime,
                    ExpressionInfixOpcode::GreaterEq => rv % prime - lv % prime,
                    ExpressionInfixOpcode::Eq => (lv % prime - rv % prime).abs(),
                    ExpressionInfixOpcode::NotEq => {
                        if lv % prime == rv % prime {
                            BigInt::one()
                        } else {
                            BigInt::zero()
                        }
                    }
                    _ => panic!("Only support comparison operators"),
                },
                _ => panic!(
                    "Unassigned variables exist: {:?}, {:?}",
                    lhs_val.unwrap().lookup_fmt(&symbolic_library.id2name),
                    rhs_val.unwrap().lookup_fmt(&symbolic_library.id2name),
                ),
            }
        }
        SymbolicValue::UnaryOp(op, expr) => {
            let error = evaluate_error_of_symbolic_value(prime, expr, assignment, symbolic_library);
            match op.0 {
                ExpressionPrefixOpcode::BoolNot => {
                    if error.is_zero() {
                        BigInt::one()
                    } else {
                        -error
                    }
                }
                _ => panic!("Only support BoolNot"),
            }
        }
        _ => todo!("{:?}", value),
    }
}

/// Accumulates the total error for a set of symbolic constraints.
///
/// This function iterates over a list of symbolic constraints and evaluates the error of each
/// constraint under a given assignment. Errors are clamped to zero to ignore negative values,
/// and the total is computed as the sum of individual errors.
///
/// # Parameters
/// - `prime`: The prime modulus used for modular arithmetic.
/// - `constraints`: A slice of symbolic value references representing the constraints.
/// - `assignment`: A mapping of symbolic names to their concrete integer values.
/// - `symbolic_library`: A mutable reference to the symbolic library providing variable lookup and
///   other symbolic operations.
///
/// # Returns
/// The total error as a `BigInt`.
pub fn accumulate_error_of_constraints(
    prime: &BigInt,
    constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> BigInt {
    constraints
        .iter()
        .map(|constraint| {
            let e =
                evaluate_error_of_symbolic_value(prime, constraint, assignment, symbolic_library);
            e.max(BigInt::zero())
        })
        .sum()
}

pub fn count_error_constraints(
    prime: &BigInt,
    constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> BigInt {
    BigInt::from(
        constraints
            .iter()
            .filter(|constraint| {
                let e = evaluate_error_of_symbolic_value(
                    prime,
                    constraint,
                    assignment,
                    symbolic_library,
                );
                e != BigInt::zero()
            })
            .count(),
    )
}

pub fn max_error_of_constraints(
    prime: &BigInt,
    constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> BigInt {
    constraints
        .iter()
        .map(|constraint| {
            let e =
                evaluate_error_of_symbolic_value(prime, constraint, assignment, symbolic_library);
            e.max(BigInt::zero())
        })
        .max()
        .unwrap_or(prime.clone())
}

/// Checks if two integers are equivalent modulo a given prime.
///
/// This function determines whether two integers are congruent modulo the specified prime,
/// accounting for potential negative values.
///
/// # Parameters
/// - `a`: The first integer.
/// - `b`: The second integer.
/// - `p`: The prime modulus.
///
/// # Returns
/// `true` if `a ≡ b (mod p)`, otherwise `false`.
pub fn is_equal_mod(a: &BigInt, b: &BigInt, p: &BigInt) -> bool {
    let mut a_mod_p = a % p;
    let mut b_mod_p = b % p;
    if a_mod_p.is_negative() {
        a_mod_p += p;
    }
    if b_mod_p.is_negative() {
        b_mod_p += p;
    }
    a_mod_p == b_mod_p
}

/// Verifies an assignment for symbolic constraints and determines whether the constraints are
/// under-constrained, over-constrained, or well-constrained.
///
/// This function evaluates both symbolic trace and side constraints for a given
/// symbolic execution environment and assignment, returning a result that categorizes the
/// verification outcome.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor (`SymbolicExecutor`) which maintains
///   the symbolic execution state and library.
/// - `symbolic_trace`: A reference to a slice of symbolic values representing the symbolic trace.
/// - `side_constraints`: A reference to a slice of symbolic values representing the side constraints.
/// - `assignment`: A mapping from symbolic names to concrete integer values representing the assignment to be verified.
/// - `setting`: Configuration settings (`BaseVerificationConfig`) including modular arithmetic parameters and template configurations.
///
/// # Returns
/// A `VerificationResult` that represents one of the following:
/// - `VerificationResult::WellConstrained`: The constraints are satisfied without ambiguity.
/// - `VerificationResult::OverConstrained`: The symbolic trace are satisfied, but the side constraints are not.
/// - `VerificationResult::UnderConstrained`: Either the program accepts unexpected input
///   (`UnderConstrainedType::UnexpectedInput`) or exhibits non-deterministic behavior
///   (`UnderConstrainedType::NonDeterministic`).
///
/// # Verification Process
/// 1. Evaluate the symbolic trace (`symbolic_trace`) and side constraints (`side_constraints`)
///    against the provided `assignment`.
/// 2. Determine the constraint status:
///     - If the symbolic trace is satisfied but the side constraints are not, return `OverConstrained`.
///     - If the side constraints are satisfied but the symbolic trace is not:
///         - Perform concrete execution with the given `assignment` to validate the symbolic
///           execution state.
///         - If the execution fails due to unexpected input, return `UnderConstrained` with details about the violated condition.
///         - If execution succeeds but produces outputs that do not match the expected assignment, return `UnderConstrained`
///           with non-deterministic output details.
///     - If both constraints are satisfied or unsatisfied in harmony, return `WellConstrained`.
///
/// # Panics
/// This function may panic if an undetermined output is encountered during execution,
/// or if the provided symbolic library lacks the expected mappings.
pub fn verify_assignment(
    sexe: &mut SymbolicExecutor,
    symbolic_trace: &[SymbolicValueRef],
    side_constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    setting: &BaseVerificationConfig,
) -> VerificationResult {
    let is_satisfy_st = evaluate_constraints(
        &setting.prime,
        symbolic_trace,
        assignment,
        &mut sexe.symbolic_library,
    );
    let is_satisfy_sc = evaluate_constraints(
        &setting.prime,
        side_constraints,
        assignment,
        &mut sexe.symbolic_library,
    );

    if is_satisfy_st && !is_satisfy_sc {
        VerificationResult::OverConstrained
    } else if !is_satisfy_st && is_satisfy_sc {
        sexe.clear();
        sexe.cur_state.add_owner(&OwnerName {
            id: sexe.symbolic_library.name2id["main"],
            counter: 0,
            access: None,
        });
        sexe.feed_arguments(
            &setting.template_param_names,
            &setting.template_param_values,
        );
        sexe.concrete_execute(&setting.target_template_name, assignment);

        if sexe.cur_state.is_failed {
            let vc = sexe.violated_condition.clone().unwrap();
            return VerificationResult::UnderConstrained(UnderConstrainedType::UnexpectedInput(
                vc.0,
                vc.1.lookup_fmt(&sexe.symbolic_library.id2name),
            ));
        }

        let mut result = VerificationResult::WellConstrained;
        for (k, v) in assignment {
            if sexe.symbolic_library.template_library
                [&sexe.symbolic_library.name2id[&setting.target_template_name]]
                .output_ids
                .contains(&k.id)
            {
                let original_sym_value = sexe.cur_state.symbol_binding_map[&k].clone();
                let mut memo = FxHashSet::default();
                let simplified_sym_value = sexe.simplify_variables(
                    &original_sym_value,
                    std::usize::MAX,
                    false,
                    false,
                    &mut memo,
                );
                let original_int_value = match simplified_sym_value {
                    SymbolicValue::ConstantInt(num) => num.clone(),
                    SymbolicValue::ConstantBool(b) => {
                        if b {
                            BigInt::one()
                        } else {
                            BigInt::zero()
                        }
                    }
                    _ => {
                        panic!(
                            "Undetermined Output: {}",
                            original_sym_value
                                .clone()
                                .lookup_fmt(&sexe.symbolic_library.id2name)
                        );
                    }
                };
                if !is_equal_mod(&original_int_value, v, &setting.prime) {
                    result = VerificationResult::UnderConstrained(
                        UnderConstrainedType::NonDeterministic(
                            k.clone(),
                            k.lookup_fmt(&sexe.symbolic_library.id2name),
                            original_int_value.clone(),
                        ),
                    );
                    break;
                }
            }
        }

        result
    } else {
        VerificationResult::WellConstrained
    }
}
