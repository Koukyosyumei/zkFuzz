use colored::Colorize;
use log::{error, trace, warn};
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Signed;
use num_traits::{One, Zero};
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use program_structure::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType,
    Statement, VariableType,
};

use crate::parser_user::{
    DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode, DebugVariableType,
    ExtendedStatement,
};
use crate::utils::{extended_euclidean, italic};

/// Simplifies a given statement by transforming certain structures into more straightforward forms.
/// Specifically, it handles inline switch operations within substitution statements.
///
/// # Arguments
///
/// * `statement` - A reference to the `Statement` to be simplified.
///
/// # Returns
///
/// A simplified `Statement`.
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

/// Represents a symbolic value used in symbolic execution, which can be a constant, variable, or an operation.
/// It supports various operations like binary, unary, conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum SymbolicValue {
    ConstantInt(BigInt),
    ConstantBool(bool),
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

/// Implements the `Debug` trait for `SymbolicValue` to provide custom formatting for debugging purposes.
impl fmt::Debug for SymbolicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolicValue::ConstantInt(value) => write!(f, "{}", value),
            SymbolicValue::ConstantBool(flag) => {
                write!(f, "{} {}", if *flag { "✅" } else { "❌" }, flag)
            }
            SymbolicValue::Variable(name) => write!(f, "{}", name),
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Eq
                | ExpressionInfixOpcode::NotEq
                | ExpressionInfixOpcode::LesserEq
                | ExpressionInfixOpcode::GreaterEq
                | ExpressionInfixOpcode::Lesser
                | ExpressionInfixOpcode::Greater => {
                    write!(f, "({} {:?} {:?})", format!("{:?}", op).green(), lhs, rhs)
                }
                ExpressionInfixOpcode::ShiftL
                | ExpressionInfixOpcode::ShiftR
                | ExpressionInfixOpcode::BitAnd
                | ExpressionInfixOpcode::BitOr
                | ExpressionInfixOpcode::BitXor => {
                    write!(f, "({} {:?} {:?})", format!("{:?}", op).red(), lhs, rhs)
                }
                _ => write!(f, "({} {:?} {:?})", format!("{:?}", op).yellow(), lhs, rhs),
            },
            SymbolicValue::Conditional(cond, if_branch, else_branch) => {
                write!(f, "({:?} {:?} {:?})", cond, if_branch, else_branch)
            }
            SymbolicValue::UnaryOp(op, expr) => match &op.0 {
                _ => write!(f, "({} {:?})", format!("{:?}", op).magenta(), expr),
            },
            SymbolicValue::Call(name, args) => {
                write!(f, "📞{}({:?})", name, args)
            }
            _ => write!(f, "❓Unknown symbolic value"),
        }
    }
}

/// Represents the access type within a symbolic expression, such as component or array access.
#[derive(Clone, Debug)]
pub enum SymbolicAccess {
    ComponentAccess(String),
    ArrayAccess(SymbolicValue),
}

impl fmt::Display for SymbolicAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.compact_fmt(f)
    }
}

impl SymbolicAccess {
    /// Provides a compact format for displaying symbolic access in expressions.
    fn compact_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            SymbolicAccess::ComponentAccess(name) => {
                write!(f, ".{}", name)
            }
            SymbolicAccess::ArrayAccess(val) => {
                write!(
                    f,
                    "[{}]",
                    format!("{:?}", val).replace("\n", "").replace("  ", " ")
                )
            }
        }
    }
}

/// Represents the state of symbolic execution, holding symbolic values,
/// trace constraints, side constraints, and depth information.
#[derive(Clone)]
pub struct SymbolicState {
    owner_name: String,
    pub template_id: String,
    depth: usize,
    pub values: HashMap<String, Box<SymbolicValue>>,
    pub trace_constraints: Vec<Box<SymbolicValue>>,
    pub side_constraints: Vec<Box<SymbolicValue>>,
}

/// Implements the `Debug` trait for `SymbolicState` to provide detailed state information during debugging.
impl fmt::Debug for SymbolicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🛠️ {}", format!("{}", "SymbolicState [").cyan())?;
        writeln!(
            f,
            "  {} {}",
            format!("👤 {}", "owner:").cyan(),
            italic(&self.owner_name).magenta()
        )?;
        writeln!(f, "  📏 {} {}", format!("{}", "depth:").cyan(), self.depth)?;
        writeln!(f, "  📋 {}", format!("{}", "values:").cyan())?;
        for (k, v) in self.values.clone().into_iter() {
            writeln!(
                f,
                "      {}: {}",
                k.replace("\n", "").replace("  ", " "),
                format!("{:?}", v.clone())
                    .replace("\n", "")
                    .replace("  ", " ")
            )?;
        }
        writeln!(
            f,
            "  {} {}",
            format!("{}", "🪶 trace_constraints:").cyan(),
            format!("{:?}", self.trace_constraints)
                .replace("\n", "")
                .replace("  ", " ")
                .replace("  ", " ")
        )?;
        writeln!(
            f,
            "  {} {}",
            format!("{}", "⛓️ side_constraints:").cyan(),
            format!("{:?}", self.side_constraints)
                .replace("\n", "")
                .replace("  ", " ")
                .replace("  ", " ")
        )?;
        write!(f, "{}", format!("{}", "]").cyan())
    }
}

impl SymbolicState {
    /// Creates a new `SymbolicState` with default values.
    pub fn new() -> Self {
        SymbolicState {
            owner_name: "".to_string(),
            template_id: "".to_string(),
            depth: 0_usize,
            values: HashMap::new(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
        }
    }

    /// Sets the owner name of the current symbolic state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the owner to set.
    pub fn set_owner(&mut self, name: String) {
        self.owner_name = name;
    }

    pub fn set_template_id(&mut self, name: String) {
        self.template_id = name;
    }

    /// Retrieves the owner name of the current symbolic state.
    ///
    /// # Returns
    ///
    /// The owner name as a string.
    pub fn get_owner(&self) -> String {
        self.owner_name.clone()
    }

    /// Sets the current depth of the symbolic state.
    ///
    /// # Arguments
    ///
    /// * `d` - The depth level to set.
    pub fn set_depth(&mut self, d: usize) {
        self.depth = d;
    }

    /// Retrieves the current depth of the symbolic state.
    ///
    /// # Returns
    ///
    /// The depth as an unsigned integer.
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Sets a symbolic value for a given variable name in the state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    /// * `value` - The symbolic value to associate with the variable.
    pub fn set_symval(&mut self, name: String, value: SymbolicValue) {
        self.values.insert(name, Box::new(value));
    }

    /// Retrieves a symbolic value associated with a given variable name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// An optional reference to the symbolic value if it exists.
    pub fn get_symval(&self, name: &str) -> Option<&Box<SymbolicValue>> {
        self.values.get(name)
    }

    /// Adds a trace constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_trace_constraint(&mut self, constraint: &SymbolicValue) {
        self.trace_constraints.push(Box::new(constraint.clone()));
    }

    /// Adds a side constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_side_constraint(&mut self, constraint: &SymbolicValue) {
        self.side_constraints.push(Box::new(constraint.clone()));
    }
}

/// Represents a symbolic template used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicTemplate {
    pub template_parameter_names: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub unrolled_outputs: HashSet<String>,
    pub var2type: HashMap<String, VariableType>,
    pub body: Vec<ExtendedStatement>,
}

/// Represents a symbolic function used in the symbolic execution process.
#[derive(Default, Clone, Debug)]
pub struct SymbolicFunction {
    pub function_argument_names: Vec<String>,
    pub body: Vec<ExtendedStatement>,
}

/// Represents a symbolic component used in the symbolic execution process.
#[derive(Default, Clone, Debug)]
pub struct SymbolicComponent {
    pub template_name: String,
    pub args: Vec<SymbolicValue>,
    pub inputs: HashMap<String, Option<SymbolicValue>>,
    pub is_done: bool,
}

/// Executes symbolic execution on a series of statements while maintaining multiple states.
/// It handles branching logic and updates constraints accordingly during execution flow.
///
/// This struct is parameterized over a lifetime `'a`, which is used for borrowing constraint statistics references.
///
/// # Fields
///
/// * `template_library` - A library storing templates for execution.
/// * `components_store` - A store for components used in execution.
/// * `variable_types` - A map storing types of variables.
/// * `prime` - A prime number used in computations.
/// * `cur_state`, `block_end_states`, `final_states` - Various states managed during execution.
/// * `max_depth` - Tracks maximum depth reached during execution.
#[derive(Clone)]
pub struct SymbolicExecutor {
    pub template_library: HashMap<String, Box<SymbolicTemplate>>,
    pub function_library: HashMap<String, Box<SymbolicFunction>>,
    pub function_counter: HashMap<String, usize>,
    pub components_store: HashMap<String, SymbolicComponent>,
    pub variable_types: HashMap<String, DebugVariableType>,
    pub prime: BigInt,
    pub propagate_substitution: bool,
    pub skip_initialization_blocks: bool,
    pub off_trace: bool,
    pub keep_track_unrolled_offset: bool,
    // states
    pub cur_state: SymbolicState,
    pub block_end_states: Vec<Box<SymbolicState>>,
    pub final_states: Vec<Box<SymbolicState>>,
    // stats
    pub max_depth: usize,
}

impl SymbolicExecutor {
    /// Creates a new instance of `SymbolicExecutor`, initializing all necessary states and statistics trackers.
    pub fn new(propagate_substitution: bool, prime: BigInt) -> Self {
        SymbolicExecutor {
            template_library: HashMap::new(),
            function_library: HashMap::new(),
            function_counter: HashMap::new(),
            components_store: HashMap::new(),
            variable_types: HashMap::new(),
            prime: prime,
            propagate_substitution: propagate_substitution,
            skip_initialization_blocks: false,
            off_trace: false,
            cur_state: SymbolicState::new(),
            block_end_states: Vec::new(),
            final_states: Vec::new(),
            max_depth: 0,
            keep_track_unrolled_offset: true,
        }
    }

    pub fn clear(&mut self) {
        self.function_counter.clear();
        self.components_store.clear();
        self.cur_state = SymbolicState::new();
        self.block_end_states.clear();
        self.final_states.clear();
        self.max_depth = 0;
    }

    // Checks if a component is ready based on its inputs being fully specified.
    //
    // # Arguments
    //
    // * 'name' - Name of the component to check readiness for.
    //
    // # Returns
    //
    // A boolean indicating readiness status.
    fn is_ready(&self, name: String) -> bool {
        self.components_store.contains_key(&name)
            && self.components_store[&name]
                .inputs
                .iter()
                .all(|(_, v)| v.is_some())
    }

    // Feeds arguments into current state variables based on provided names and expressions.
    //
    // # Arguments
    //
    // * 'names' : Vector containing names corresponding with expressions being fed as arguments.
    // * 'args' : Vector containing expressions whose evaluated results will be assigned as argument values.
    pub fn feed_arguments(&mut self, names: &Vec<String>, args: &Vec<Expression>) {
        for (n, a) in names.iter().zip(args.iter()) {
            let evaled_a = self.evaluate_expression(&DebugExpression(a.clone()));
            self.cur_state.set_symval(
                format!("{}.{}", self.cur_state.get_owner(), n.to_string()),
                evaled_a,
            );
        }
    }

    // Registers library template by extracting input signals from block statement body provided along with template parameter names list.
    //
    // # Arguments
    //
    // * 'name' : Name under which template will be registered within library .
    // * 'body' : Block statement serving as main logic body defining behavior captured by template .
    // * 'template_parameter_names': List containing names identifying parameters used within template logic .
    pub fn register_library(
        &mut self,
        name: String,
        body: Statement,
        template_parameter_names: &Vec<String>,
    ) {
        let mut inputs: Vec<String> = vec![];
        let mut outputs: Vec<String> = vec![];
        let mut var2type: HashMap<String, VariableType> = HashMap::new();
        match &body {
            Statement::Block { stmts, .. } => {
                for s in stmts {
                    if let Statement::InitializationBlock {
                        initializations, ..
                    } = s.clone()
                    {
                        for init in initializations {
                            if let Statement::Declaration { name, xtype, .. } = init.clone() {
                                var2type.insert(name.clone(), xtype.clone());
                                if let VariableType::Signal(typ, _taglist) = xtype.clone() {
                                    match typ {
                                        SignalType::Input => {
                                            inputs.push(name);
                                        }
                                        SignalType::Output => {
                                            outputs.push(name);
                                        }
                                        SignalType::Intermediate => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                warn!("Cannot Find Block Statement");
            }
        }

        let template = SymbolicTemplate {
            template_parameter_names: template_parameter_names.clone(),
            inputs: inputs,
            outputs: outputs,
            unrolled_outputs: HashSet::new(),
            var2type: var2type,
            body: vec![
                ExtendedStatement::DebugStatement(body),
                ExtendedStatement::Ret,
            ],
        };
        self.template_library.insert(name, Box::new(template));
    }

    pub fn register_function(
        &mut self,
        name: String,
        body: Statement,
        function_argument_names: &Vec<String>,
    ) {
        self.function_library.insert(
            name.clone(),
            Box::new(SymbolicFunction {
                function_argument_names: function_argument_names.clone(),
                body: vec![
                    ExtendedStatement::DebugStatement(body),
                    ExtendedStatement::Ret,
                ],
            }),
        );
        self.function_counter.insert(name.clone(), 0_usize);
    }

    /// Expands all stack states by executing each statement block recursively,
    /// updating depth and managing branching paths in execution flow.
    ///
    /// # Arguments
    ///
    /// * `statements` - A vector of extended statements to execute symbolically.
    /// * `cur_bid` - Current block index being executed.
    /// * `depth` - Current depth level in execution flow for tracking purposes.
    fn expand_all_stack_states(
        &mut self,
        statements: &Vec<ExtendedStatement>,
        cur_bid: usize,
        depth: usize,
    ) {
        let stack_states = self.block_end_states.clone();
        self.block_end_states.clear();
        for state in &stack_states.clone() {
            self.cur_state = *state.clone();
            self.cur_state.set_depth(depth);
            self.execute(statements, cur_bid);
        }
    }

    /// Executes a sequence of statements symbolically from a specified starting block index,
    /// updating internal states and handling control structures like if-else and loops appropriately.
    ///
    /// # Arguments
    ///
    /// * `statements` - A vector of extended statements representing program logic to execute symbolically.
    /// * `cur_bid` - Current block index to start execution from.
    pub fn execute(&mut self, statements: &Vec<ExtendedStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.max_depth = max(self.max_depth, self.cur_state.get_depth());
            match &statements[cur_bid] {
                ExtendedStatement::DebugStatement(stmt) => {
                    match stmt {
                        Statement::InitializationBlock {
                            initializations,
                            xtype,
                            ..
                        } => {
                            let mut is_input = false;
                            if let VariableType::Signal(SignalType::Input, _taglist) = xtype.clone()
                            {
                                is_input = true;
                            }

                            if !(self.skip_initialization_blocks && is_input) {
                                for init in initializations {
                                    self.execute(
                                        &vec![ExtendedStatement::DebugStatement(init.clone())],
                                        0,
                                    );
                                }
                            }
                            self.block_end_states = vec![Box::new(self.cur_state.clone())];
                            self.expand_all_stack_states(
                                statements,
                                cur_bid + 1,
                                self.cur_state.get_depth(),
                            );
                        }
                        Statement::Block { meta, stmts, .. } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
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
                        Statement::IfThenElse {
                            meta,
                            cond,
                            if_case,
                            else_case,
                            ..
                        } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            let tmp_cond = self.evaluate_expression(&DebugExpression(cond.clone()));
                            let original_evaled_condition = self.fold_variables(&tmp_cond, true);
                            let evaled_condition =
                                self.fold_variables(&tmp_cond, !self.propagate_substitution);

                            // Save the current state
                            let cur_depth = self.cur_state.get_depth();
                            let stack_states = self.block_end_states.clone();

                            // Create a branch in the symbolic state
                            let mut if_state = self.cur_state.clone();
                            let mut else_state = self.cur_state.clone();

                            if let SymbolicValue::ConstantBool(false) = evaled_condition {
                                if !self.off_trace {
                                    trace!(
                                        "{}",
                                        format!(
                                            "(elem_id={}) 🚧 Unreachable `Then` Branch",
                                            meta.elem_id
                                        )
                                        .yellow()
                                    );
                                }
                            } else {
                                if_state.push_trace_constraint(&evaled_condition);
                                if_state.push_side_constraint(&original_evaled_condition);
                                if_state.set_depth(cur_depth + 1);
                                self.cur_state = if_state.clone();
                                self.execute(
                                    &vec![ExtendedStatement::DebugStatement(*if_case.clone())],
                                    0,
                                );
                                self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                            }

                            if let Some(else_stmt) = else_case {
                                let mut stack_states_if_true = self.block_end_states.clone();
                                self.block_end_states = stack_states;
                                let neg_evaled_condition =
                                    if let SymbolicValue::ConstantBool(v) = evaled_condition {
                                        SymbolicValue::ConstantBool(!v)
                                    } else {
                                        SymbolicValue::UnaryOp(
                                            DebugExpressionPrefixOpcode(
                                                ExpressionPrefixOpcode::BoolNot,
                                            ),
                                            Box::new(evaled_condition),
                                        )
                                    };
                                let original_neg_evaled_condition =
                                    if let SymbolicValue::ConstantBool(v) =
                                        original_evaled_condition
                                    {
                                        SymbolicValue::ConstantBool(!v)
                                    } else {
                                        SymbolicValue::UnaryOp(
                                            DebugExpressionPrefixOpcode(
                                                ExpressionPrefixOpcode::BoolNot,
                                            ),
                                            Box::new(original_evaled_condition),
                                        )
                                    };
                                if let SymbolicValue::ConstantBool(false) = neg_evaled_condition {
                                    if !self.off_trace {
                                        trace!(
                                            "{}",
                                            format!(
                                                "(elem_id={}) 🚧 Unreachable `Else` Branch",
                                                meta.elem_id
                                            )
                                            .yellow()
                                        );
                                    }
                                } else {
                                    else_state.push_trace_constraint(&neg_evaled_condition);
                                    else_state.push_side_constraint(&original_neg_evaled_condition);
                                    else_state.set_depth(cur_depth + 1);
                                    self.cur_state = else_state;
                                    self.execute(
                                        &vec![ExtendedStatement::DebugStatement(
                                            *else_stmt.clone(),
                                        )],
                                        0,
                                    );
                                    self.expand_all_stack_states(
                                        statements,
                                        cur_bid + 1,
                                        cur_depth,
                                    );
                                }
                                self.block_end_states.append(&mut stack_states_if_true);
                            }
                        }
                        Statement::While {
                            meta, cond, stmt, ..
                        } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            // Symbolic execution of loops is complex. This is a simplified approach.
                            let tmp_cond = self.evaluate_expression(&DebugExpression(cond.clone()));
                            let evaled_condition =
                                self.fold_variables(&tmp_cond, !self.propagate_substitution);

                            if let SymbolicValue::ConstantBool(flag) = evaled_condition {
                                if flag {
                                    self.execute(
                                        &vec![ExtendedStatement::DebugStatement(*stmt.clone())],
                                        0,
                                    );
                                    self.block_end_states.pop();
                                    self.execute(statements, cur_bid);
                                } else {
                                    self.block_end_states.push(Box::new(self.cur_state.clone()));
                                }
                            } else {
                                panic!("This tool currently cannot handle the symbolic condition of While Loop: {:?}", evaled_condition);
                            }

                            self.expand_all_stack_states(
                                statements,
                                cur_bid + 1,
                                self.cur_state.get_depth(),
                            );
                            // Note: This doesn't handle loop invariants or fixed-point computation
                        }
                        Statement::Return { meta, value, .. } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            let tmp_val = self.evaluate_expression(&DebugExpression(value.clone()));
                            let return_value =
                                self.fold_variables(&tmp_val, !self.propagate_substitution);
                            // Handle return value (e.g., store in a special "return" variable)
                            self.cur_state.set_symval(
                                format!("{}.__return__", self.cur_state.get_owner()).to_string(),
                                return_value,
                            );
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Declaration {
                            name,
                            dimensions,
                            xtype,
                            ..
                        } => {
                            let var_name = if dimensions.is_empty() {
                                format!("{}.{}", self.cur_state.get_owner(), name.clone())
                            } else {
                                //"todo".to_string()
                                format!(
                                    "{}.{}<{:?}>",
                                    self.cur_state.get_owner(),
                                    name,
                                    &dimensions
                                        .iter()
                                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                                        .collect::<Vec<_>>()
                                )
                            };
                            self.variable_types
                                .insert(name.to_string(), DebugVariableType(xtype.clone()));
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
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            let expr = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            let original_value = self.fold_variables(&expr, true);
                            let value = self.fold_variables(&expr, !self.propagate_substitution);

                            let var_name = if access.is_empty() {
                                format!("{}.{}", self.cur_state.get_owner(), var.clone())
                            } else {
                                //format!("{}", var)
                                format!(
                                    "{}.{}{}",
                                    self.cur_state.get_owner(),
                                    var,
                                    &access
                                        .iter()
                                        .map(|arg0: &Access| self.evaluate_access(&arg0.clone(),))
                                        .map(|debug_access| debug_access.to_string())
                                        .collect::<Vec<_>>()
                                        .join("")
                                )
                            };

                            if self.keep_track_unrolled_offset {
                                if self.template_library[&self.cur_state.template_id]
                                    .var2type
                                    .contains_key(&var.clone())
                                {
                                    if let VariableType::Signal(SignalType::Output, _) = self
                                        .template_library[&self.cur_state.template_id]
                                        .var2type[&var.clone()]
                                    {
                                        self.template_library
                                            .get_mut(&self.cur_state.template_id)
                                            .unwrap()
                                            .unrolled_outputs
                                            .insert(var_name.clone());
                                    }
                                }
                            }

                            self.cur_state.set_symval(var_name.clone(), value.clone());

                            if !access.is_empty() {
                                for acc in access {
                                    if let Access::ComponentAccess(tmp_name) = acc {
                                        if let Some(component) =
                                            self.components_store.get_mut(var.as_str())
                                        {
                                            component
                                                .inputs
                                                .insert(tmp_name.clone(), Some(value.clone()));
                                        }
                                    }
                                }

                                if self.is_ready(var.to_string()) {
                                    if !self.components_store[var].is_done {
                                        let mut subse = SymbolicExecutor::new(
                                            self.propagate_substitution,
                                            self.prime.clone(),
                                        );

                                        subse.template_library = self.template_library.clone();
                                        subse.cur_state.set_owner(format!(
                                            "{}.{}",
                                            self.cur_state.get_owner(),
                                            var.clone()
                                        ));

                                        let templ = &self.template_library
                                            [&self.components_store[var].template_name];
                                        subse.cur_state.set_template_id(
                                            self.components_store[var].template_name.clone(),
                                        );

                                        for i in 0..(templ.template_parameter_names.len()) {
                                            subse.cur_state.set_symval(
                                                format!(
                                                    "{}.{}",
                                                    subse.cur_state.get_owner(),
                                                    templ.template_parameter_names[i]
                                                ),
                                                self.components_store[var].args[i].clone(),
                                            );
                                        }

                                        for (k, v) in
                                            self.components_store[var].inputs.clone().into_iter()
                                        {
                                            subse.cur_state.set_symval(
                                                format!("{}.{}", subse.cur_state.get_owner(), k),
                                                v.unwrap(),
                                            );
                                        }

                                        if !self.off_trace {
                                            trace!(
                                                "{}",
                                                format!("{}", "===========================").cyan()
                                            );
                                            trace!(
                                                "📞 Call {}",
                                                self.components_store[var].template_name
                                            );
                                        }

                                        subse.execute(&templ.body, 0);

                                        if subse.final_states.len() > 1 {
                                            warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                                        }
                                        let mut sub_trace_constraints =
                                            subse.final_states[0].trace_constraints.clone();
                                        let mut sub_side_constraints =
                                            subse.final_states[0].side_constraints.clone();
                                        self.cur_state
                                            .trace_constraints
                                            .append(&mut sub_trace_constraints);
                                        self.cur_state
                                            .side_constraints
                                            .append(&mut sub_side_constraints);
                                        if !self.off_trace {
                                            trace!(
                                                "{}",
                                                format!("{}", "===========================").cyan()
                                            );
                                        }
                                    }
                                }
                            }

                            match value {
                                SymbolicValue::Call(callee_name, args) => {
                                    // Initializing the Template Component
                                    let mut comp_inputs: HashMap<String, Option<SymbolicValue>> =
                                        HashMap::new();
                                    for inp_name in
                                        &self.template_library[&callee_name].inputs.clone()
                                    {
                                        comp_inputs.insert(inp_name.clone(), None);
                                    }
                                    let c = SymbolicComponent {
                                        template_name: callee_name.clone(),
                                        args: args.clone(),
                                        inputs: comp_inputs,
                                        is_done: false,
                                    };
                                    self.components_store.insert(var.to_string(), c);
                                }
                                _ => {
                                    if self.variable_types[var].0 != VariableType::Var {
                                        let cont = SymbolicValue::BinaryOp(
                                            Box::new(SymbolicValue::Variable(var_name.clone())),
                                            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                            Box::new(value),
                                        );
                                        self.cur_state.push_trace_constraint(&cont);

                                        if let AssignOp::AssignConstraintSignal = op {
                                            let original_cont = SymbolicValue::BinaryOp(
                                                Box::new(SymbolicValue::Variable(var_name.clone())),
                                                DebugExpressionInfixOpcode(
                                                    ExpressionInfixOpcode::Eq,
                                                ),
                                                Box::new(original_value),
                                            );
                                            self.cur_state.push_side_constraint(&original_cont);
                                        }
                                    }
                                }
                            }

                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::MultSubstitution {
                            meta, lhe, op, rhe, ..
                        } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }

                            let lhe_val = self.evaluate_expression(&DebugExpression(lhe.clone()));
                            let rhe_val = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            let simple_lhs = self.fold_variables(&lhe_val, true);
                            let lhs = self.fold_variables(&lhe_val, !self.propagate_substitution);
                            let simple_rhs = self.fold_variables(&rhe_val, true);
                            let rhs = self.fold_variables(&rhe_val, !self.propagate_substitution);

                            // Handle multiple substitution (simplified)
                            let cont = SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                Box::new(rhs),
                            );
                            self.cur_state.push_trace_constraint(&cont);
                            if let AssignOp::AssignConstraintSignal = op {
                                // Handle multiple substitution (simplified)
                                let simple_cont = SymbolicValue::BinaryOp(
                                    Box::new(simple_lhs),
                                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                    Box::new(simple_rhs),
                                );
                                self.cur_state.push_side_constraint(&simple_cont);
                            }
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::ConstraintEquality { meta, lhe, rhe } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }

                            let lhe_val = self.evaluate_expression(&DebugExpression(lhe.clone()));
                            let rhe_val = self.evaluate_expression(&DebugExpression(rhe.clone()));
                            let original_lhs = self.fold_variables(&lhe_val, true);
                            let lhs = self.fold_variables(&lhe_val, !self.propagate_substitution);
                            let original_rhs = self.fold_variables(&rhe_val, true);
                            let rhs = self.fold_variables(&rhe_val, !self.propagate_substitution);

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

                            self.cur_state.push_trace_constraint(&cond);
                            self.cur_state.push_side_constraint(&original_cond);

                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::Assert { meta, arg, .. } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            let expr = self.evaluate_expression(&DebugExpression(arg.clone()));
                            let condition =
                                self.fold_variables(&expr, !self.propagate_substitution);
                            self.cur_state.push_trace_constraint(&condition);
                            self.execute(statements, cur_bid + 1);
                        }
                        Statement::UnderscoreSubstitution {
                            meta,
                            op: _,
                            rhe: _,
                            ..
                        } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            // Underscore substitution doesn't affect the symbolic state
                        }
                        Statement::LogCall { meta, args: _, .. } => {
                            if !self.off_trace {
                                trace!("(elem_id={}) {:?}", meta.elem_id, self.cur_state);
                            }
                            // Logging doesn't affect the symbolic state
                        }
                    }
                }
                ExtendedStatement::Ret => {
                    if !self.off_trace {
                        trace!("{} {:?}", format!("{}", "🔙 Ret:").red(), self.cur_state);
                    }
                    self.final_states.push(Box::new(self.cur_state.clone()));
                }
            }
        } else {
            self.block_end_states.push(Box::new(self.cur_state.clone()));
        }
    }

    pub fn concrete_execute(
        &mut self,
        id: &String,
        assignment: &HashMap<String, BigInt>,
        off_trace: bool,
    ) {
        self.cur_state.template_id = id.to_string();
        for (vname, value) in assignment.into_iter() {
            self.cur_state
                .set_symval(vname.clone(), SymbolicValue::ConstantInt(value.clone()));
        }
        /*
        for arg in &self.template_library[id].inputs {
            let vname = format!("{}.{}", self.cur_state.get_owner(), arg.to_string());
            self.cur_state.set_symval(
                vname.clone(),
                SymbolicValue::ConstantInt(assignment[&vname].clone()),
            );
        }*/

        self.skip_initialization_blocks = true;
        self.off_trace = off_trace;
        self.execute(&self.template_library[id].body.clone(), 0);
    }

    /// Evaluates a symbolic access expression, converting it into a `SymbolicAccess` value.
    ///
    /// # Arguments
    ///
    /// * `access` - The `Access` to evaluate.
    ///
    /// # Returns
    ///
    /// A `SymbolicAccess` representing the evaluated access.
    fn evaluate_access(&mut self, access: &Access) -> SymbolicAccess {
        match &access {
            Access::ComponentAccess(name) => SymbolicAccess::ComponentAccess(name.clone()),
            Access::ArrayAccess(expr) => {
                let tmp_e = self.evaluate_expression(&DebugExpression(expr.clone()));
                SymbolicAccess::ArrayAccess(self.fold_variables(&tmp_e, false))
            }
        }
    }

    fn fold_variables(
        &self,
        symval: &SymbolicValue,
        only_constatant_folding: bool,
    ) -> SymbolicValue {
        match &symval {
            SymbolicValue::Variable(name) => {
                let original_var_name = if let Some(last_part) = name.split('.').last() {
                    last_part.clone()
                } else {
                    &name.clone()
                };
                if only_constatant_folding {
                    if let Some(boxed_value) = self.cur_state.get_symval(&name) {
                        if let SymbolicValue::ConstantInt(v) = *boxed_value.clone() {
                            return SymbolicValue::ConstantInt(v);
                        }
                    }
                    if self.template_library[&self.cur_state.template_id]
                        .var2type
                        .contains_key(&original_var_name.to_string())
                    {
                        let typ = self.template_library[&self.cur_state.template_id].var2type
                            [&original_var_name.to_string()]
                            .clone();
                        if let VariableType::Var = typ {
                            return *self.cur_state.get_symval(&name).cloned().unwrap_or_else(
                                || Box::new(SymbolicValue::Variable(name.to_string())),
                            );
                        }
                    }
                    symval.clone()
                } else {
                    *self
                        .cur_state
                        .get_symval(&name)
                        .cloned()
                        .unwrap_or_else(|| Box::new(SymbolicValue::Variable(name.to_string())))
                }
            }
            SymbolicValue::BinaryOp(lv, infix_op, rv) => {
                let lhs = self.fold_variables(lv, only_constatant_folding);
                let rhs = self.fold_variables(rv, only_constatant_folding);
                match (&lhs, &rhs) {
                    (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                        match &infix_op.0 {
                            ExpressionInfixOpcode::Add => {
                                SymbolicValue::ConstantInt((lv + rv) % self.prime.clone())
                            }
                            ExpressionInfixOpcode::Sub => {
                                SymbolicValue::ConstantInt((lv - rv) % self.prime.clone())
                            }
                            ExpressionInfixOpcode::Mul => {
                                SymbolicValue::ConstantInt((lv * rv) % self.prime.clone())
                            }
                            ExpressionInfixOpcode::Div => {
                                let mut r = self.prime.clone();
                                let mut new_r = rv.clone();
                                if r.is_negative() {
                                    r += self.prime.clone();
                                }
                                if new_r.is_negative() {
                                    new_r += self.prime.clone();
                                }

                                let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
                                rv_inv %= self.prime.clone();
                                if rv_inv.is_negative() {
                                    rv_inv += self.prime.clone();
                                }

                                SymbolicValue::ConstantInt((lv * rv_inv) % self.prime.clone())
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
                            ExpressionInfixOpcode::Lesser => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() < rv % self.prime.clone(),
                            ),
                            ExpressionInfixOpcode::Greater => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() > rv % self.prime.clone(),
                            ),
                            ExpressionInfixOpcode::LesserEq => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() <= rv % self.prime.clone(),
                            ),
                            ExpressionInfixOpcode::GreaterEq => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() >= rv % self.prime.clone(),
                            ),
                            ExpressionInfixOpcode::Eq => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() == rv % self.prime.clone(),
                            ),
                            ExpressionInfixOpcode::NotEq => SymbolicValue::ConstantBool(
                                lv % self.prime.clone() != rv % self.prime.clone(),
                            ),
                            _ => SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                infix_op.clone(),
                                Box::new(rhs),
                            ),
                        }
                    }
                    (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => {
                        match &infix_op.0 {
                            ExpressionInfixOpcode::BoolAnd => {
                                SymbolicValue::ConstantBool(*lv && *rv)
                            }
                            ExpressionInfixOpcode::BoolOr => {
                                SymbolicValue::ConstantBool(*lv || *rv)
                            }
                            _ => SymbolicValue::BinaryOp(
                                Box::new(lhs),
                                infix_op.clone(),
                                Box::new(rhs),
                            ),
                        }
                    }
                    _ => SymbolicValue::BinaryOp(Box::new(lhs), infix_op.clone(), Box::new(rhs)),
                }
            }
            SymbolicValue::Conditional(cond, then_val, else_val) => SymbolicValue::Conditional(
                Box::new(self.fold_variables(cond, only_constatant_folding)),
                Box::new(self.fold_variables(then_val, only_constatant_folding)),
                Box::new(self.fold_variables(else_val, only_constatant_folding)),
            ),
            SymbolicValue::UnaryOp(prefix_op, value) => {
                let folded_symval = self.fold_variables(value, only_constatant_folding);
                match &folded_symval {
                    SymbolicValue::ConstantInt(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Box::new(folded_symval)),
                    },
                    SymbolicValue::ConstantBool(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Box::new(folded_symval)),
                    },
                    _ => SymbolicValue::UnaryOp(prefix_op.clone(), Box::new(folded_symval)),
                }
            }
            SymbolicValue::Array(elements) => SymbolicValue::Array(
                elements
                    .iter()
                    .map(|e| self.fold_variables(e, only_constatant_folding))
                    .collect(),
            ),
            SymbolicValue::Tuple(elements) => SymbolicValue::Tuple(
                elements
                    .iter()
                    .map(|e| self.fold_variables(e, only_constatant_folding))
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => SymbolicValue::UniformArray(
                Box::new(self.fold_variables(element, only_constatant_folding)),
                Box::new(self.fold_variables(count, only_constatant_folding)),
            ),
            SymbolicValue::Call(func_name, args) => SymbolicValue::Call(
                func_name.clone(),
                args.iter()
                    .map(|arg| self.fold_variables(arg, only_constatant_folding))
                    .collect(),
            ),
            _ => symval.clone(),
        }
    }

    /// Evaluates a symbolic expression, converting it into a `SymbolicValue`.
    ///
    /// This function handles various types of expressions, including constants, variables,
    /// and complex operations. It recursively evaluates sub-expressions as needed.
    ///
    /// # Arguments
    ///
    /// * `expr` - The `DebugExpression` to evaluate.

    ///
    /// # Returns
    ///
    /// A `SymbolicValue` representing the evaluated expression.
    fn evaluate_expression(&mut self, expr: &DebugExpression) -> SymbolicValue {
        match &expr.0 {
            Expression::Number(_meta, value) => {
                SymbolicValue::ConstantInt(value.clone() % self.prime.clone())
            }
            Expression::Variable {
                name,
                access,
                meta: _,
            } => {
                let resolved_name = if access.is_empty() {
                    format!("{}.{}", self.cur_state.get_owner(), name.clone())
                } else {
                    format!(
                        "{}.{}{}",
                        self.cur_state.get_owner(),
                        name,
                        &access
                            .iter()
                            .map(|arg0: &Access| self.evaluate_access(&arg0.clone(),))
                            .map(|debug_access| debug_access.to_string())
                            .collect::<Vec<_>>()
                            .join("")
                    )
                };
                SymbolicValue::Variable(resolved_name)
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
                let tmp_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(&DebugExpression(arg.clone())))
                    .collect();
                let evaluated_args = tmp_args
                    .iter()
                    .map(|arg| self.fold_variables(&arg, false))
                    .collect();
                if self.template_library.contains_key(id) {
                    SymbolicValue::Call(id.clone(), evaluated_args)
                } else if self.function_library.contains_key(id) {
                    let mut subse =
                        SymbolicExecutor::new(self.propagate_substitution, self.prime.clone());
                    subse.cur_state.set_owner(format!(
                        "{}.{}.{}",
                        self.cur_state.get_owner(),
                        id.clone(),
                        self.function_counter[id]
                    ));
                    subse.template_library = self.template_library.clone();
                    subse.function_library = self.function_library.clone();
                    subse.function_counter = self.function_counter.clone();

                    let func = &self.function_library[id];
                    for i in 0..(func.function_argument_names.len()) {
                        subse.cur_state.set_symval(
                            format!(
                                "{}.{}",
                                subse.cur_state.get_owner(),
                                func.function_argument_names[i]
                            ),
                            evaluated_args[i].clone(),
                        );
                    }

                    if !self.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                        trace!("📞 Call {}", id);
                    }

                    subse.execute(&func.body, 0);

                    if subse.final_states.len() > 1 {
                        warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                    }
                    let mut sub_trace_constraints = subse.final_states[0].trace_constraints.clone();
                    let mut sub_side_constraints = subse.final_states[0].side_constraints.clone();
                    self.cur_state
                        .trace_constraints
                        .append(&mut sub_trace_constraints);
                    self.cur_state
                        .side_constraints
                        .append(&mut sub_side_constraints);
                    if !self.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    self.function_counter
                        .insert(id.to_string(), self.function_counter[id] + 1);

                    *subse.final_states[0].values
                        [&format!("{}.__return__", subse.final_states[0].get_owner()).to_string()]
                        .clone()
                } else {
                    error!("Unknown Callee: {}", id);
                    SymbolicValue::Call(id.clone(), evaluated_args)
                }
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
                error!("Unhandled expression type: {:?}", expr);
                SymbolicValue::Variable(format!("Unhandled({:?})", expr))
            }
        }
    }
}
