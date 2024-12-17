use std::fmt;
use std::io::Write;
use std::rc::Rc;

use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Signed;
use num_traits::Zero;
use rustc_hash::{FxHashMap, FxHashSet};

use program_structure::ast::Expression;
use program_structure::ast::ExpressionInfixOpcode;
use program_structure::ast::ExpressionPrefixOpcode;

use crate::executor::symbolic_execution::{SymbolicExecutor, SymbolicExecutorSetting};
use crate::executor::symbolic_value::{
    OwnerName, SymbolicLibrary, SymbolicName, SymbolicValue, SymbolicValueRef,
};
use crate::executor::utils::{extended_euclidean, modpow};

pub enum UnderConstrainedType {
    UnusedOutput,
    Deterministic,
    NonDeterministic,
}

/// Represents the result of a constraint verification process.
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
                UnderConstrainedType::NonDeterministic => {
                    "🔥 UnderConstrained (Non-Deterministic) 🔥".red().bold()
                }
            },
            VerificationResult::OverConstrained => "💣 OverConstrained 💣".yellow().bold(),
            VerificationResult::WellConstrained => "✅ WellConstrained ✅".green().bold(),
        };
        write!(f, "{output}")
    }
}

/// Represents a counterexample when constraints are found to be invalid.
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
    pub quick_mode: bool,
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
        SymbolicValue::Variable(name) => {
            variables.insert(name.clone());
        }
        SymbolicValue::Assign(lhs, rhs) | SymbolicValue::AssignEq(lhs, rhs) => {
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
        _ => {}
    }
}

pub fn get_dependency_graph(
    values: &[SymbolicValueRef],
    graph: &mut FxHashMap<SymbolicName, FxHashSet<SymbolicName>>,
) {
    for value in values {
        match value.as_ref() {
            SymbolicValue::Assign(lhs, rhs) | SymbolicValue::AssignEq(lhs, rhs) => {
                if let SymbolicValue::Variable(name) = lhs.as_ref() {
                    graph.entry(name.clone()).or_default();
                    extract_variables_from_symbolic_value(&rhs, graph.get_mut(&name).unwrap());
                } else {
                    panic!("Left hand of the assignment is not a variable");
                }
            }
            SymbolicValue::BinaryOp(lhs, op, rhs) => {
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
            SymbolicValue::UnaryOp(op, expr) => {
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
            SymbolicValue::Assign(lhs, rhs) | SymbolicValue::AssignEq(lhs, rhs) => {
                if let SymbolicValue::Variable(name) = lhs.as_ref() {
                    let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
                    if let SymbolicValue::ConstantInt(num) = &rhs_val {
                        assignment.insert(name.clone(), num.clone());
                    } else {
                        success = false;
                    }
                } else {
                    panic!("Left hand of the assignment is not a variable");
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
                            _ => panic!("Non-Boolean Operation"),
                        }
                    }
                    (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => {
                        match &op.0 {
                            ExpressionInfixOpcode::BoolAnd => *lv && *rv,
                            ExpressionInfixOpcode::BoolOr => *lv || *rv,
                            _ => todo!(),
                        }
                    }
                    _ => panic!("Unassigned variables exist"),
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
                        _ => panic!("Unassigned variables exist"),
                    },
                    _ => panic!("Non-Boolean Operation"),
                };
                if !flag {
                    success = false;
                }
            }
            _ => panic!("Non-Supported SymbolicValue"),
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
        SymbolicValue::Variable(name) => {
            SymbolicValue::ConstantInt(assignment.get(name).unwrap().clone())
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
        SymbolicValue::Assign(lhs, rhs) | SymbolicValue::AssignEq(lhs, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val, &rhs_val) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                    SymbolicValue::ConstantBool(lv % prime == rv % prime)
                }
                _ => panic!("Unassigned variables exist"),
            }
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => {
            let lhs_val = evaluate_symbolic_value(prime, lhs, assignment, symbolic_library);
            let rhs_val = evaluate_symbolic_value(prime, rhs, assignment, symbolic_library);
            match (&lhs_val, &rhs_val) {
                (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match op.0 {
                    ExpressionInfixOpcode::Add => SymbolicValue::ConstantInt((lv + rv) % prime),
                    ExpressionInfixOpcode::Sub => SymbolicValue::ConstantInt((lv - rv) % prime),
                    ExpressionInfixOpcode::Mul => SymbolicValue::ConstantInt((lv * rv) % prime),
                    ExpressionInfixOpcode::Pow => SymbolicValue::ConstantInt(modpow(lv, rv, prime)),
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
                    _ => todo!("{:?}", value),
                },
                (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => match &op.0 {
                    ExpressionInfixOpcode::BoolAnd => SymbolicValue::ConstantBool(*lv && *rv),
                    ExpressionInfixOpcode::BoolOr => SymbolicValue::ConstantBool(*lv || *rv),
                    _ => todo!("{:?}", value),
                },
                _ => panic!("Unassigned variables exist"),
            }
        }
        SymbolicValue::UnaryOp(op, expr) => {
            let expr_val = evaluate_symbolic_value(prime, expr, assignment, symbolic_library);
            match &expr_val {
                SymbolicValue::ConstantInt(rv) => match op.0 {
                    ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                    _ => panic!("Unassigned variables exist"),
                },
                SymbolicValue::ConstantBool(rv) => match op.0 {
                    ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                    _ => panic!("Unassigned variables exist"),
                },
                _ => todo!("{:?}", value),
            }
        }
        SymbolicValue::Call(id, args) => {
            let setting = SymbolicExecutorSetting {
                prime: prime.clone(),
                propagate_substitution: false,
                skip_initialization_blocks: false,
                only_initialization_blocks: false,
                off_trace: true,
                keep_track_constraints: false,
                substitute_output: false,
            };
            let mut subse = SymbolicExecutor::new(symbolic_library, &setting);

            let func = subse.symbolic_library.function_library[id].clone();
            for i in 0..(func.function_argument_names.len()) {
                let sname = SymbolicName {
                    name: func.function_argument_names[i],
                    owner: subse.cur_state.owner_name.clone(),
                    access: None,
                };
                subse.cur_state.set_rc_symval(
                    sname,
                    Rc::new(evaluate_symbolic_value(
                        prime,
                        &args[i],
                        assignment,
                        subse.symbolic_library,
                    )),
                );
            }
            subse.execute(&func.body.clone(), 0);

            if !subse.symbolic_store.final_states.is_empty() {
                let return_name = SymbolicName {
                    name: usize::MAX,
                    owner: subse.symbolic_store.final_states[0].owner_name.clone(),
                    access: None,
                };
                let return_value =
                    (*subse.symbolic_store.final_states[0].values[&return_name].clone()).clone();
                return_value
            } else {
                panic!("Empty Final State");
            }
        }
        _ => todo!("{:?}", value),
    }
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
        return VerificationResult::OverConstrained;
    } else if !is_satisfy_tc && is_satisfy_sc {
        sexe.clear();
        sexe.cur_state.add_owner(&OwnerName {
            name: sexe.symbolic_library.name2id["main"],
            counter: 0,
            access: None,
        });
        sexe.feed_arguments(
            &setting.template_param_names,
            &setting.template_param_values,
        );
        sexe.concrete_execute(&setting.id, assignment);

        let mut flag = false;
        if sexe.symbolic_store.final_states.len() > 0 {
            for (k, v) in assignment {
                if sexe.symbolic_library.template_library
                    [&sexe.symbolic_library.name2id[&setting.id]]
                    .outputs
                    .contains(&k.name)
                {
                    let unboxed_value = &sexe.symbolic_store.final_states[0].values[&k];
                    if let SymbolicValue::ConstantInt(num) = &(*unboxed_value.clone()) {
                        if *num != *v {
                            flag = true;
                            break;
                        }
                    } else {
                        panic!(
                            "Undetermined Output: {}",
                            unboxed_value
                                .clone()
                                .lookup_fmt(&sexe.symbolic_library.id2name)
                        );
                    }
                }
            }
        }

        if flag {
            return VerificationResult::UnderConstrained(UnderConstrainedType::NonDeterministic);
        } else {
            return VerificationResult::WellConstrained;
        }
    } else {
        return VerificationResult::WellConstrained;
    }
}
