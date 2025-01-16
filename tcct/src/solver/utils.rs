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
use serde_json::{json, Value};

use crate::executor::debug_ast::{
    DebuggableExpressionInfixOpcode, DebuggableExpressionPrefixOpcode,
};
use crate::executor::symbolic_execution::SymbolicExecutor;
use crate::executor::symbolic_setting::SymbolicExecutorSetting;
use crate::executor::symbolic_value::{
    evaluate_binary_op, OwnerName, SymbolicLibrary, SymbolicName, SymbolicValue, SymbolicValueRef,
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
    assignment: &mut FxHashMap<SymbolicName, BigInt>,
    symbolic_library: &mut SymbolicLibrary,
) -> (bool, usize) {
    let mut success = true;
    let mut failure_pos = 0;
    for (i, inst) in trace.iter().enumerate() {
        match inst.as_ref() {
            SymbolicValue::ConstantBool(b) => {
                if !b {
                    success = false;
                    failure_pos = i;
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
                    SymbolicValue::ConstantBool(rv) => match op.0 {
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
            _ => panic!(
                "A constraint should be one of `ConstantBool`, `Assign`, `AssignEq`, `AssignCall`, `BinaryOp` and `UnaryOp`. Found: {}",
                inst.lookup_fmt(&symbolic_library.id2name)
            ),
        }
    }

    (success, failure_pos)
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
            DebuggableExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
            expr,
        ) => match (*expr.clone()).clone() {
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Lesser => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Greater),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::Greater => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::LesserEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::GreaterEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::LesserEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::Eq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::NotEq),
                    rhs.clone(),
                ),
                ExpressionInfixOpcode::NotEq => SymbolicValue::BinaryOp(
                    lhs.clone(),
                    DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
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
    symbolic_trace: &[SymbolicValueRef],
    side_constraints: &[SymbolicValueRef],
    assignment: &FxHashMap<SymbolicName, BigInt>,
    setting: &BaseVerificationConfig,
) -> VerificationResult {
    let is_satisfy_tc = evaluate_constraints(
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
                let original_sym_value = &sexe.cur_state.symbol_binding_map[&k];
                let original_int_value = match &**original_sym_value {
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
