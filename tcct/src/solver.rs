use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Signed;
use num_traits::{One, Zero};
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;

use crate::symbolic_execution::SymbolicExecutor;
use crate::symbolic_value::SymbolicName;
use crate::symbolic_value::{OwnerName, SymbolicValue};
use crate::utils::extended_euclidean;

/// Represents the result of a constraint verification process.
pub enum VerificationResult {
    UnderConstrained,
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
            VerificationResult::UnderConstrained => "🔥 UnderConstrained 🔥".red().bold(),
            VerificationResult::OverConstrained => "💣 OverConstrained 💣".yellow().bold(),
            VerificationResult::WellConstrained => "✅ WellConstrained ✅".green().bold(),
        };
        write!(f, "{output}")
    }
}

/// Represents a counterexample when constraints are found to be invalid.
pub struct CounterExample {
    flag: VerificationResult,
    assignment: FxHashMap<SymbolicName, BigInt>,
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

        for (var, value) in &self.assignment {
            s += &format!("{}", "║".red());
            s += &format!(
                "           {} {} = {} \n",
                "➡️".cyan(),
                var.lookup_fmt(lookup).magenta().bold(),
                value.to_string().bright_yellow()
            );
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
fn is_vulnerable(vr: &VerificationResult) -> bool {
    match vr {
        VerificationResult::UnderConstrained => true,
        VerificationResult::OverConstrained => true,
        VerificationResult::WellConstrained => false,
    }
}

/// Configures the settings for the verification process.
pub struct VerificationSetting {
    pub id: String,
    pub prime: BigInt,
    pub quick_mode: bool,
    pub progress_interval: usize,
    pub template_param_names: Vec<String>,
    pub template_param_values: Vec<Expression>,
}

/// Performs a brute-force search over variable assignments to evaluate constraints.
///
/// # Parameters
/// - `sexe`: A mutable reference to the symbolic executor.
/// - `trace_constraints`: A vector of constraints representing the program trace.
/// - `side_constraints`: A vector of additional constraints for validation.
/// - `setting`: The verification settings.
///
/// # Returns
/// An `Option<CounterExample>` containing a counterexample if constraints are invalid, or `None` otherwise.
pub fn brute_force_search(
    sexe: &mut SymbolicExecutor,
    trace_constraints: &Vec<Rc<SymbolicValue>>,
    side_constraints: &Vec<Rc<SymbolicValue>>,
    setting: &VerificationSetting,
) -> Option<CounterExample> {
    let mut trace_variables = extract_variables(trace_constraints);
    let mut side_variables = extract_variables(side_constraints);

    let mut variables = Vec::new();
    variables.append(&mut trace_variables);
    variables.append(&mut side_variables);
    let variables_set: HashSet<SymbolicName> = variables.iter().cloned().collect();
    variables = variables_set.into_iter().collect();

    let mut assignment = FxHashMap::default();
    let current_iteration = Arc::new(AtomicUsize::new(0));

    fn search(
        sexe: &mut SymbolicExecutor,
        trace_constraints: &[Rc<SymbolicValue>],
        side_constraints: &[Rc<SymbolicValue>],
        setting: &VerificationSetting,
        index: usize,
        variables: &[SymbolicName],
        assignment: &mut FxHashMap<SymbolicName, BigInt>,
        current_iteration: &Arc<AtomicUsize>,
    ) -> VerificationResult {
        if index == variables.len() {
            let iter = current_iteration.fetch_add(1, Ordering::SeqCst);
            if iter % setting.progress_interval == 0 {
                print!(
                    "\rProgress: {} / {}^{}",
                    iter,
                    &setting.prime,
                    variables.len()
                );
                io::stdout().flush().unwrap();
            }

            let is_satisfy_tc = evaluate_constraints(&setting.prime, trace_constraints, assignment);
            let is_satisfy_sc = evaluate_constraints(&setting.prime, side_constraints, assignment);

            if is_satisfy_tc && !is_satisfy_sc {
                return VerificationResult::OverConstrained;
            } else if !is_satisfy_tc && is_satisfy_sc {
                sexe.clear();
                sexe.cur_state.add_owner(&OwnerName {
                    name: sexe.symbolic_library.name2id["main"],
                    counter: 0,
                });
                sexe.feed_arguments(
                    &setting.template_param_names,
                    &setting.template_param_values,
                );
                sexe.concrete_execute(&setting.id, assignment);

                let mut flag = false;
                if sexe.symbolic_store.final_states.len() > 0 {
                    for vname in &sexe.symbolic_library.template_library
                        [&sexe.symbolic_library.name2id[&setting.id]]
                        .unrolled_outputs
                    {
                        //let vname = format!("{}.{}", sexe.cur_state.get_owner(), n.to_string());
                        let unboxed_value =
                            sexe.symbolic_store.final_states[0].values[&vname.clone()].clone();
                        if let SymbolicValue::ConstantInt(v) = (*unboxed_value.clone()).clone() {
                            if v != assignment[&vname.clone()] {
                                flag = true;
                                break;
                            }
                        }
                    }
                }

                if flag {
                    return VerificationResult::UnderConstrained;
                } else {
                    return VerificationResult::WellConstrained;
                }
            } else {
                return VerificationResult::WellConstrained;
            }
        }

        let var = &variables[index];
        if setting.quick_mode {
            let candidates = vec![BigInt::zero(), BigInt::one(), -1 * BigInt::one()];
            for c in candidates.into_iter() {
                assignment.insert(var.clone(), c.clone());
                let result = search(
                    sexe,
                    trace_constraints,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );
                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(var);
            }
        } else {
            let mut value = BigInt::zero();
            while value < setting.prime {
                assignment.insert(var.clone(), value.clone());
                let result = search(
                    sexe,
                    trace_constraints,
                    side_constraints,
                    setting,
                    index + 1,
                    variables,
                    assignment,
                    current_iteration,
                );
                if is_vulnerable(&result) {
                    return result;
                }
                assignment.remove(var);
                value += BigInt::one();
            }
        }
        VerificationResult::WellConstrained
    }

    let flag = search(
        sexe,
        &trace_constraints,
        &side_constraints,
        setting,
        0,
        &variables,
        &mut assignment,
        &current_iteration,
    );

    print!(
        "\rProgress: {} / {}^{}",
        current_iteration.load(Ordering::SeqCst),
        setting.prime,
        variables.len()
    );
    io::stdout().flush().unwrap();

    println!(
        "\nSearch completed. Total iterations: {}",
        current_iteration.load(Ordering::SeqCst)
    );

    if is_vulnerable(&flag) {
        Some(CounterExample {
            flag: flag,
            assignment: assignment,
        })
    } else {
        None
    }
}

/// Extracts all unique variable names referenced in a set of constraints.
///
/// # Parameters
/// - `constraints`: A slice of symbolic values representing the constraints.
///
/// # Returns
/// A vector of unique `SymbolicName`s referenced in the constraints.
fn extract_variables(constraints: &[Rc<SymbolicValue>]) -> Vec<SymbolicName> {
    let mut variables = Vec::new();
    for constraint in constraints {
        extract_variables_from_symbolic_value(constraint, &mut variables);
    }
    //variables.sort();
    variables.dedup();
    variables
}

/// Recursively extracts variable names from a symbolic value.
///
/// # Parameters
/// - `value`: The `SymbolicValue` to analyze.
/// - `variables`: A mutable reference to a vector where extracted variable names will be stored.
fn extract_variables_from_symbolic_value(value: &SymbolicValue, variables: &mut Vec<SymbolicName>) {
    match value {
        SymbolicValue::Variable(name) => variables.push(name.clone()),
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
        _ => {}
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
fn evaluate_constraints(
    prime: &BigInt,
    constraints: &[Rc<SymbolicValue>],
    assignment: &FxHashMap<SymbolicName, BigInt>,
) -> bool {
    constraints.iter().all(|constraint| {
        let sv = evaluate_symbolic_value(prime, constraint, assignment);
        match sv {
            SymbolicValue::ConstantBool(b) => b,
            _ => panic!("Non-bool output value is detected when evaluating a constraint"),
        }
    })
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
fn count_satisfied_constraints(
    prime: &BigInt,
    constraints: &[Rc<SymbolicValue>],
    assignment: &FxHashMap<SymbolicName, BigInt>,
) -> usize {
    constraints
        .iter()
        .filter(|constraint| {
            let sv = evaluate_symbolic_value(prime, constraint, assignment);
            match sv {
                SymbolicValue::ConstantBool(b) => b,
                _ => panic!("Non-bool output value is detected when evaluating a constraint"),
            }
        })
        .count()
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
fn evaluate_symbolic_value(
    prime: &BigInt,
    value: &SymbolicValue,
    assignment: &FxHashMap<SymbolicName, BigInt>,
) -> SymbolicValue {
    match value {
        SymbolicValue::ConstantBool(_b) => value.clone(),
        SymbolicValue::ConstantInt(_v) => value.clone(),
        SymbolicValue::Variable(name) => {
            SymbolicValue::ConstantInt(assignment.get(name).unwrap().clone())
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment);
            match (&lhs_val, &rhs_val) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match op.0 {
                    ExpressionInfixOpcode::Add => SymbolicValue::ConstantInt((lv + rv) % prime),
                    ExpressionInfixOpcode::Sub => SymbolicValue::ConstantInt((lv - rv) % prime),
                    ExpressionInfixOpcode::Mul => SymbolicValue::ConstantInt((lv * rv) % prime),
                    ExpressionInfixOpcode::Div => {
                        if rv.is_zero() {
                            SymbolicValue::ConstantInt(BigInt::zero())
                        } else {
                            let mut r = prime.clone();
                            let mut new_r = rv.clone();
                            if r.is_negative() {
                                r += prime;
                            }
                            if new_r.is_negative() {
                                new_r += prime;
                            }

                            let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
                            rv_inv %= prime;
                            if rv_inv.is_negative() {
                                rv_inv += prime;
                            }

                            SymbolicValue::ConstantInt((lv * rv_inv) % prime)
                        }
                    }
                    ExpressionInfixOpcode::IntDiv => SymbolicValue::ConstantInt(lv / rv),
                    ExpressionInfixOpcode::Mod => SymbolicValue::ConstantInt(lv % rv),
                    ExpressionInfixOpcode::BitOr => SymbolicValue::ConstantInt(lv | rv),
                    ExpressionInfixOpcode::BitAnd => SymbolicValue::ConstantInt(lv & rv),
                    ExpressionInfixOpcode::BitXor => SymbolicValue::ConstantInt(lv ^ rv),
                    ExpressionInfixOpcode::ShiftL => {
                        SymbolicValue::ConstantInt(lv << rv.to_usize().unwrap())
                    }
                    ExpressionInfixOpcode::ShiftR => {
                        SymbolicValue::ConstantInt(lv >> rv.to_usize().unwrap())
                    }
                    ExpressionInfixOpcode::Lesser => {
                        SymbolicValue::ConstantBool(lv % prime < rv % prime)
                    }
                    ExpressionInfixOpcode::Greater => {
                        SymbolicValue::ConstantBool(lv % prime > rv % prime)
                    }
                    ExpressionInfixOpcode::LesserEq => {
                        SymbolicValue::ConstantBool(lv % prime <= rv % prime)
                    }
                    ExpressionInfixOpcode::GreaterEq => {
                        SymbolicValue::ConstantBool(lv % prime >= rv % prime)
                    }
                    ExpressionInfixOpcode::Eq => {
                        SymbolicValue::ConstantBool(lv % prime == rv % prime)
                    }
                    ExpressionInfixOpcode::NotEq => {
                        SymbolicValue::ConstantBool(lv % prime != rv % prime)
                    }
                    _ => todo!(),
                },
                (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => match &op.0 {
                    ExpressionInfixOpcode::BoolAnd => SymbolicValue::ConstantBool(*lv && *rv),
                    ExpressionInfixOpcode::BoolOr => SymbolicValue::ConstantBool(*lv || *rv),
                    _ => todo!(),
                },
                _ => panic!("Unassigned variables exist"),
            }
        }
        SymbolicValue::UnaryOp(op, expr) => {
            let expr_val = evaluate_symbolic_value(prime, expr, assignment);
            match &expr_val {
                SymbolicValue::ConstantInt(rv) => match op.0 {
                    ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                    _ => panic!("Unassigned variables exist"),
                },
                SymbolicValue::ConstantBool(rv) => match op.0 {
                    ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                    _ => panic!("Unassigned variables exist"),
                },
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}
