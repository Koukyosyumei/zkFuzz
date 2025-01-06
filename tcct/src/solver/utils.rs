use std::fmt;
use std::io::Write;
use std::rc::Rc;

use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::ToPrimitive;
use num_traits::{One, Signed, Zero};
use rustc_hash::{FxHashMap, FxHashSet};

use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;

use crate::executor::debug_ast::{DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode};
use crate::executor::symbolic_execution::{SymbolicExecutor, SymbolicExecutorSetting};
use crate::executor::symbolic_value::{
    evaluate_binary_op, OwnerName, SymbolicLibrary, SymbolicName, SymbolicValue, SymbolicValueRef,
};

#[derive(Clone)]
pub enum UnderConstrainedType {
    UnusedOutput,
    Deterministic,
    NonDeterministic(String, BigInt),
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
                    "👻 UnderConstrained (Unused-Output) 👻".red().bold()
                }
                UnderConstrainedType::Deterministic => {
                    "🧟 UnderConstrained (Deterministic) 🧟".red().bold()
                }
                UnderConstrainedType::NonDeterministic(name, value) => format!(
                    "🔥 UnderConstrained (Non-Deterministic) 🔥\n║           ➡️ `{}` is expected to be `{}`",
                    name, value
                )
                .red()
                .bold(),
            },
            VerificationResult::OverConstrained => "💣 OverConstrained 💣".yellow().bold(),
            VerificationResult::WellConstrained => "✅ WellConstrained ✅".green().bold(),
        };
        write!(f, "{output}")
    }
}

/// Represents a counterexample when constraints are found to be invalid.
#[derive(Clone)]
pub struct CounterExample {
    pub flag: VerificationResult,
    pub assignment: FxHashMap<SymbolicName, BigInt>,
}

impl CounterExample {
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

        for (var_name, value) in &self.assignment {
            if var_name.owner.len() == 1 {
                s += &format!("{}", "║".red());
                s += &format!(
                    "           {} {} = {} \n",
                    "➡️".cyan(),
                    var_name.lookup_fmt(lookup).magenta().bold(),
                    value.to_string().bright_yellow()
                );
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
pub struct VerificationSetting {
    pub id: String,
    pub prime: BigInt,
    pub range: BigInt,
    pub quick_mode: bool,
    pub heuristics_mode: bool,
    pub progress_interval: usize,
    pub template_param_names: Vec<String>,
    pub template_param_values: Vec<Expression>,
}

/// Extracts all unique variable names referenced in a set of constraints.
///
/// # Parameters
/// - `constraints`: A slice of symbolic values representing the constraints.
///
/// # Returns
/// A vector of unique `SymbolicName`s referenced in the constraints.
pub fn extract_variables(constraints: &[SymbolicValueRef]) -> Vec<SymbolicName> {
    let mut variables = FxHashSet::default();
    for constraint in constraints {
        extract_variables_from_symbolic_value(constraint, &mut variables);
    }
    variables.into_iter().collect()
}

/// Recursively extracts variable names from a symbolic value.
///
/// # Parameters
/// - `value`: The `SymbolicValue` to analyze.
/// - `variables`: A mutable reference to a vector where extracted variable names will be stored.
pub fn extract_variables_from_symbolic_value(
    value: &SymbolicValue,
    variables: &mut FxHashSet<SymbolicName>,
) {
    match value {
        SymbolicValue::Variable(sym_name) => {
            variables.insert(sym_name.clone());
        }
        SymbolicValue::Assign(lhs, rhs, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            extract_variables_from_symbolic_value(&lhs, variables);
            extract_variables_from_symbolic_value(&rhs, variables);
        }
        SymbolicValue::BinaryOp(lhs, _, rhs) => {
            extract_variables_from_symbolic_value(&lhs, variables);
            extract_variables_from_symbolic_value(&rhs, variables);
        }
        SymbolicValue::Conditional(cond, if_true, if_false) => {
            extract_variables_from_symbolic_value(&cond, variables);
            extract_variables_from_symbolic_value(&if_true, variables);
            extract_variables_from_symbolic_value(&if_false, variables);
        }
        SymbolicValue::UnaryOp(_, expr) => extract_variables_from_symbolic_value(&expr, variables),
        SymbolicValue::Array(elements) | SymbolicValue::Tuple(elements) => {
            for elem in elements {
                extract_variables_from_symbolic_value(&elem, variables);
            }
        }
        SymbolicValue::UniformArray(value, size) => {
            extract_variables_from_symbolic_value(&value, variables);
            extract_variables_from_symbolic_value(&size, variables);
        }
        SymbolicValue::Call(_, args) => {
            for arg in args {
                extract_variables_from_symbolic_value(&arg, variables);
            }
        }
        SymbolicValue::Conditional(cond, then_val, else_val) => {
            extract_variables_from_symbolic_value(&cond, variables);
            extract_variables_from_symbolic_value(&then_val, variables);
            extract_variables_from_symbolic_value(&else_val, variables);
        }
        _ => {}
    }
}

pub fn get_dependency_graph(
    values: &[SymbolicValueRef],
    graph: &mut FxHashMap<SymbolicName, FxHashSet<SymbolicName>>,
) {
    for value in values {
        match value.as_ref() {
            SymbolicValue::Assign(lhs, rhs, _)
            | SymbolicValue::AssignEq(lhs, rhs)
            | SymbolicValue::AssignCall(lhs, rhs, _) => {
                if let SymbolicValue::Variable(sym_name) = lhs.as_ref() {
                    graph.entry(sym_name.clone()).or_default();
                    extract_variables_from_symbolic_value(&rhs, graph.get_mut(&sym_name).unwrap());
                } else {
                    panic!("Left hand of the assignment is not a variable");
                }
            }
            SymbolicValue::BinaryOp(lhs, _op, rhs) => {
                let mut variables = FxHashSet::default();
                extract_variables_from_symbolic_value(&lhs, &mut variables);
                extract_variables_from_symbolic_value(&rhs, &mut variables);

                for v1 in &variables {
                    for v2 in &variables {
                        if v1 != v2 {
                            graph.entry(v1.clone()).or_default().insert(v2.clone());
                            graph.entry(v2.clone()).or_default().insert(v1.clone());
                        }
                    }
                }
            }
            SymbolicValue::UnaryOp(_op, expr) => {
                let mut variables = FxHashSet::default();
                extract_variables_from_symbolic_value(&expr, &mut variables);
                for v1 in &variables {
                    for v2 in &variables {
                        if v1 != v2 {
                            graph.entry(v1.clone()).or_default().insert(v2.clone());
                            graph.entry(v2.clone()).or_default().insert(v1.clone());
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }
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
                SymbolicValue::ConstantBool(b) => b,
                _ => panic!("Non-bool output value is detected when evaluating a constraint"),
            }
        })
    }
}

/// Counts the number of satisfied constraints given a variable assignment.
///
/// # Parameters
/// - `prime`: The prime modulus for computations.
/// - `constraints`: A slice of symbolic values representing the constraints to evaluate.
/// - `assignment`: A hash map of variable assignments.
///
/// # Returns
/// The number of satisfied constraints.
pub fn count_satisfied_constraints(
    prime: &BigInt,
    constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> usize {
    constraints
        .iter()
        .filter(|constraint| {
            let sv = evaluate_symbolic_value(prime, constraint, assignment, symbolic_library);
            match sv {
                SymbolicValue::ConstantBool(b) => b,
                _ => panic!("Non-bool output value is detected when evaluating a constraint"),
            }
        })
        .count()
}

pub fn emulate_symbolic_values(
    prime: &BigInt,
    values: &[SymbolicValueRef],
    assignment: &mut FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> bool {
    let mut success = true;
    for value in values {
        match value.as_ref() {
            SymbolicValue::ConstantBool(b) => {
                if !b {
                    success = false;
                }
            }
            SymbolicValue::Assign(lhs, rhs, _)
            | SymbolicValue::AssignEq(lhs, rhs)
            | SymbolicValue::AssignCall(lhs, rhs, _) => {
                if let SymbolicValue::Variable(sym_name) = lhs.as_ref() {
                    let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
                    match &rhs_val {
                        SymbolicValue::ConstantInt(num) => {
                            assignment.insert(sym_name.clone(), num.clone());
                        }
                        SymbolicValue::ConstantBool(b) => {
                            assignment.insert(
                                sym_name.clone(),
                                if *b { BigInt::one() } else { BigInt::zero() },
                            );
                        }
                        _ => {
                            success = false;
                        }
                    }
                } else {
                    panic!(
                        "Left hand of the assignment is not a variable: {}",
                        value.lookup_fmt(&symbolic_library.id2name)
                    );
                }
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) => {
                let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
                let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
                let flag = match (&lhs_val, &rhs_val) {
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
                                value.lookup_fmt(&symbolic_library.id2name)
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
                        value.lookup_fmt(&symbolic_library.id2name)
                    ),
                };
                if !flag {
                    success = false;
                }
            }
            SymbolicValue::UnaryOp(op, expr) => {
                let expr_val = evaluate_symbolic_value(prime, expr, assignment, symbolic_library);
                let flag = match &expr_val {
                    SymbolicValue::ConstantBool(rv) => match op.0 {
                        ExpressionPrefixOpcode::BoolNot => !rv,
                        _ => panic!(
                            "Unassigned variables exist: {}",
                            value.lookup_fmt(&symbolic_library.id2name)
                        ),
                    },
                    _ => panic!(
                        "Non-Boolean Operation: {}",
                        value.lookup_fmt(&symbolic_library.id2name)
                    ),
                };
                if !flag {
                    success = false;
                }
            }
            _ => panic!(
                "A constraint should be one of `ConstantBool`, `Assign`, `AssignEq`, `AssignCall`, `BinaryOp` and `UnaryOp`. Found: {}",
                value.lookup_fmt(&symbolic_library.id2name)
            ),
        }
    }
    return success;
}

/// Evaluates a symbolic value given a variable assignment.
///
/// # Parameters
/// - `prime`: The prime modulus for computations.
/// - `value`: The `SymbolicValue` to evaluate.
/// - `assignment`: A hash map of variable assignments.
///
/// # Returns
/// The evaluated `SymbolicValue`.
pub fn evaluate_symbolic_value(
    prime: &BigInt,
    value: &SymbolicValue,
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> SymbolicValue {
    match value {
        SymbolicValue::ConstantBool(_b) => value.clone(),
        SymbolicValue::ConstantInt(_v) => value.clone(),
        SymbolicValue::Variable(sym_name) => {
            if !assignment.contains_key(sym_name) {
                panic!(
                    "name={} is not available in the assignment",
                    sym_name.lookup_fmt(&symbolic_library.id2name)
                );
            }
            SymbolicValue::ConstantInt(assignment.get(sym_name).unwrap().clone())
        }
        SymbolicValue::Array(elements) => SymbolicValue::Array(
            elements
                .iter()
                .map(|e| {
                    Rc::new(evaluate_symbolic_value(
                        prime,
                        e,
                        assignment,
                        symbolic_library,
                    ))
                })
                .collect(),
        ),
        SymbolicValue::UniformArray(elem, counts) => {
            let evaled_elem = evaluate_symbolic_value(prime, elem, assignment, symbolic_library);
            let evaled_counts =
                evaluate_symbolic_value(prime, counts, assignment, symbolic_library);
            if let SymbolicValue::ConstantInt(c) = evaled_counts {
                SymbolicValue::Array(vec![Rc::new(evaled_elem); c.to_usize().unwrap()])
            } else {
                SymbolicValue::UniformArray(Rc::new(evaled_elem), Rc::new(evaled_counts))
            }
        }
        SymbolicValue::Assign(lhs, rhs, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val, &rhs_val) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                    SymbolicValue::ConstantBool(lv % prime == rv % prime)
                }
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantBool(rv)) => {
                    let rv_to_int = if *rv { BigInt::one() } else { BigInt::zero() };
                    SymbolicValue::ConstantBool(lv % prime == rv_to_int)
                }
                _ => panic!(
                    "Unassigned variables exist: {}",
                    value.lookup_fmt(&symbolic_library.id2name)
                ),
            }
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            evaluate_binary_op(&lhs_val, &rhs_val, &prime, &op)
        }
        SymbolicValue::UnaryOp(op, expr) => {
            let expr_val = evaluate_symbolic_value(prime, expr, assignment, symbolic_library);
            match &expr_val {
                SymbolicValue::ConstantInt(rv) => match op.0 {
                    ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                    _ => panic!(
                        "Unassigned variables exist: {}",
                        value.lookup_fmt(&symbolic_library.id2name)
                    ),
                },
                SymbolicValue::ConstantBool(rv) => match op.0 {
                    ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
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
            match &cond_val {
                SymbolicValue::ConstantBool(true) => then_val,
                SymbolicValue::ConstantBool(false) => else_val,
                _ => panic!(
                    "Unassigned variables exist: {}",
                    cond_val.lookup_fmt(&symbolic_library.id2name)
                ),
            }
        }
        SymbolicValue::Call(id, args) => {
            let setting = SymbolicExecutorSetting {
                prime: prime.clone(),
                skip_initialization_blocks: false,
                only_initialization_blocks: false,
                off_trace: true,
                keep_track_constraints: false,
                substitute_output: false,
                propagate_assignments: true,
            };
            let mut subse = SymbolicExecutor::new(symbolic_library, &setting);

            let func = subse.symbolic_library.function_library[id].clone();
            for i in 0..(func.function_argument_names.len()) {
                let sym_name = SymbolicName::new(
                    func.function_argument_names[i],
                    subse.cur_state.owner_name.clone(),
                    None,
                );
                subse.cur_state.set_rc_sym_val(
                    sym_name,
                    Rc::new(evaluate_symbolic_value(
                        prime,
                        &args[i],
                        assignment,
                        subse.symbolic_library,
                    )),
                );
            }
            subse.execute(&func.body.clone(), 0);

            let return_name =
                SymbolicName::new(usize::MAX, subse.cur_state.owner_name.clone(), None);
            let return_value = (*subse.cur_state.symbol_binding_map[&return_name].clone()).clone();
            return_value
        }
        _ => todo!("{:?}", value),
    }
}

pub fn flip_op(value: &SymbolicValue) -> SymbolicValue {
    match value {
        SymbolicValue::UnaryOp(
            DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
            expr,
        ) => match (*expr.clone()).clone() {
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Lesser => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Greater),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::Greater => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::LesserEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::GreaterEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::LesserEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::Eq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::NotEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::NotEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                    rhs.clone(),
                ),
                _ => value.clone(),
            },
            _ => value.clone(),
        },
        _ => value.clone(),
    }
}

pub fn evaluate_error_of_symbolic_value(
    prime: &BigInt,
    value: &SymbolicValue,
    assignment: &FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> BigInt {
    match value {
        SymbolicValue::ConstantBool(b) => {
            if *b {
                BigInt::zero()
            } else {
                BigInt::one()
            }
        }
        SymbolicValue::Assign(lhs, rhs, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val, &rhs_val) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                    (lv % prime - rv % prime).abs()
                }
                _ => panic!("Unassigned variables exist"),
            }
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val, &rhs_val) {
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
                    lhs_val.lookup_fmt(&symbolic_library.id2name),
                    rhs_val.lookup_fmt(&symbolic_library.id2name),
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

fn is_equal_mod(a: &BigInt, b: &BigInt, p: &BigInt) -> bool {
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

pub fn verify_assignment(
    sexe: &mut SymbolicExecutor,
    trace_constraints: &[SymbolicValueRef],
    side_constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    setting: &VerificationSetting,
) -> VerificationResult {
    let is_satisfy_tc = evaluate_constraints(
        &setting.prime,
        trace_constraints,
        assignment,
        &mut sexe.symbolic_library,
    );
    let is_satisfy_sc = evaluate_constraints(
        &setting.prime,
        side_constraints,
        assignment,
        &mut sexe.symbolic_library,
    );

    if is_satisfy_tc && !is_satisfy_sc {
        VerificationResult::OverConstrained
    } else if !is_satisfy_tc && is_satisfy_sc {
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
        sexe.concrete_execute(&setting.id, assignment);

        if sexe.cur_state.is_failed {
            return VerificationResult::UnderConstrained(UnderConstrainedType::Deterministic);
        }

        let mut result = VerificationResult::WellConstrained;
        for (k, v) in assignment {
            if sexe.symbolic_library.template_library[&sexe.symbolic_library.name2id[&setting.id]]
                .output_ids
                .contains(&k.id)
            {
                let original_sym_value = &sexe.cur_state.symbol_binding_map[&k];
                let original_int_value = match &(*original_sym_value.clone()) {
                    SymbolicValue::ConstantInt(num) => num.clone(),
                    SymbolicValue::ConstantBool(b) => {
                        if *b {
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
