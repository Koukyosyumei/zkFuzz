use colored::Colorize;
use log::{error, trace, warn};
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Signed;
use num_traits::{One, Zero};
use rustc_hash::FxHashMap;
use std::cmp::max;
use std::collections::HashSet;
use std::fmt;

use program_structure::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType,
    Statement, VariableType,
};

use crate::parser_user::{
    DebugAccess, DebugAssignOp, DebugExpression, DebugExpressionInfixOpcode,
    DebugExpressionPrefixOpcode, DebugStatement, DebugVariableType,
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
        Statement::While { meta, cond, stmt } => Statement::While {
            meta: meta.clone(),
            cond: cond.clone(),
            stmt: Box::new(simplify_statement(stmt)),
        },
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

/// Represents the access type within a symbolic expression, such as component or array access.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SymbolicAccess {
    ComponentAccess(usize),
    ArrayAccess(SymbolicValue),
}

impl SymbolicAccess {
    /// Provides a compact format for displaying symbolic access in expressions.
    fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        match &self {
            SymbolicAccess::ComponentAccess(name) => {
                format!(".{}", lookup[name])
            }
            SymbolicAccess::ArrayAccess(val) => {
                format!(
                    "[{}]",
                    val.lookup_fmt(lookup).replace("\n", "").replace("  ", " ")
                )
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnerName {
    name: usize,
    counter: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolicName {
    pub name: usize,
    pub owner: Vec<OwnerName>,
    pub access: Vec<SymbolicAccess>,
}

impl SymbolicName {
    fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        format!(
            "{}.{}{}",
            self.owner
                .iter()
                .map(|e: &OwnerName| lookup[&e.name].clone())
                .collect::<Vec<_>>()
                .join("."),
            lookup[&self.name].clone(),
            self.access
                .iter()
                .map(|s: &SymbolicAccess| s.lookup_fmt(lookup))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

/// Represents a symbolic value used in symbolic execution, which can be a constant, variable, or an operation.
/// It supports various operations like binary, unary, conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum SymbolicValue {
    ConstantInt(BigInt),
    ConstantBool(bool),
    Variable(SymbolicName),
    BinaryOp(
        Box<SymbolicValue>,
        DebugExpressionInfixOpcode,
        Box<SymbolicValue>,
    ),
    Conditional(Box<SymbolicValue>, Box<SymbolicValue>, Box<SymbolicValue>),
    UnaryOp(DebugExpressionPrefixOpcode, Box<SymbolicValue>),
    Array(Vec<Box<SymbolicValue>>),
    Tuple(Vec<Box<SymbolicValue>>),
    UniformArray(Box<SymbolicValue>, Box<SymbolicValue>),
    Call(usize, Vec<Box<SymbolicValue>>),
}

/// Implements the `Debug` trait for `SymbolicValue` to provide custom formatting for debugging purposes.
impl SymbolicValue {
    fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        match self {
            SymbolicValue::ConstantInt(value) => format!("{}", value),
            SymbolicValue::ConstantBool(flag) => {
                format!("{} {}", if *flag { "✅" } else { "❌" }, flag)
            }
            SymbolicValue::Variable(sname) => sname.lookup_fmt(lookup),
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Eq
                | ExpressionInfixOpcode::NotEq
                | ExpressionInfixOpcode::LesserEq
                | ExpressionInfixOpcode::GreaterEq
                | ExpressionInfixOpcode::Lesser
                | ExpressionInfixOpcode::Greater => {
                    format!(
                        "({} {} {})",
                        format!("{:?}", op).green(),
                        lhs.lookup_fmt(lookup),
                        rhs.lookup_fmt(lookup)
                    )
                }
                ExpressionInfixOpcode::ShiftL
                | ExpressionInfixOpcode::ShiftR
                | ExpressionInfixOpcode::BitAnd
                | ExpressionInfixOpcode::BitOr
                | ExpressionInfixOpcode::BitXor
                | ExpressionInfixOpcode::Div
                | ExpressionInfixOpcode::IntDiv => {
                    format!(
                        "({} {} {})",
                        format!("{:?}", op).red(),
                        lhs.lookup_fmt(lookup),
                        rhs.lookup_fmt(lookup)
                    )
                }
                _ => format!(
                    "({} {} {})",
                    format!("{:?}", op).yellow(),
                    lhs.lookup_fmt(lookup),
                    rhs.lookup_fmt(lookup)
                ),
            },
            SymbolicValue::Conditional(cond, if_branch, else_branch) => {
                format!(
                    "({} {} {})",
                    cond.lookup_fmt(lookup),
                    if_branch.lookup_fmt(lookup),
                    else_branch.lookup_fmt(lookup)
                )
            }
            SymbolicValue::UnaryOp(op, expr) => match &op.0 {
                _ => format!(
                    "({} {})",
                    format!("{:?}", op).magenta(),
                    expr.lookup_fmt(lookup)
                ),
            },
            SymbolicValue::Call(name, args) => {
                format!(
                    "📞{}({})",
                    name,
                    args.into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::Array(elems) => {
                format!(
                    "🧬 {}",
                    elems
                        .into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::UniformArray(elem, counts) => {
                format!(
                    "🧬 ({}, {})",
                    elem.lookup_fmt(lookup),
                    counts.lookup_fmt(lookup)
                )
            }
            _ => format!("❓Unknown symbolic value"),
        }
    }
}

/// Represents the state of symbolic execution, holding symbolic values,
/// trace constraints, side constraints, and depth information.
#[derive(Clone)]
pub struct SymbolicState {
    owner_name: Vec<OwnerName>,
    pub template_id: usize,
    depth: usize,
    pub values: FxHashMap<SymbolicName, Box<SymbolicValue>>,
    pub trace_constraints: Vec<Box<SymbolicValue>>,
    pub side_constraints: Vec<Box<SymbolicValue>>,
}

/// Implements the `Debug` trait for `SymbolicState` to provide detailed state information during debugging.
impl SymbolicState {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        let mut s = "".to_string();
        s += &format!("🛠️ {}", format!("{}", "SymbolicState [\n").cyan());
        s += &format!(
            "  {} {}\n",
            format!("👤 {}", "owner:").cyan(),
            italic(&format!(
                "{:?}",
                &self
                    .owner_name
                    .iter()
                    .map(|c: &OwnerName| lookup[&c.name].clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .magenta()
        );
        s += &format!("  📏 {} {}\n", format!("{}", "depth:").cyan(), self.depth);
        s += &format!("  📋 {}\n", format!("{}", "values:").cyan());
        for (k, v) in self.values.iter() {
            s += &format!(
                "      {}: {}\n",
                k.lookup_fmt(lookup),
                format!("{}", v.lookup_fmt(lookup))
                    .replace("\n", "")
                    .replace("  ", " ")
            );
        }
        s += &format!(
            "  {} {}\n",
            format!("{}", "🪶 trace_constraints:").cyan(),
            format!(
                "{}",
                &self
                    .trace_constraints
                    .iter()
                    .map(|c| c.lookup_fmt(lookup))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace("\n", "")
            .replace("  ", " ")
            .replace("  ", " ")
        );
        s += &format!(
            "  {} {}\n",
            format!("{}", "⛓️ side_constraints:").cyan(),
            format!(
                "{}",
                &self
                    .side_constraints
                    .iter()
                    .map(|c| c.lookup_fmt(lookup))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace("\n", "")
            .replace("  ", " ")
            .replace("  ", " ")
        );
        s += &format!("{}\n", format!("{}", "]").cyan());
        s
    }
}

impl SymbolicState {
    /// Creates a new `SymbolicState` with default values.
    pub fn new() -> Self {
        SymbolicState {
            owner_name: Vec::new(),
            template_id: usize::MAX,
            depth: 0_usize,
            values: FxHashMap::default(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
        }
    }

    /// Sets the owner name of the current symbolic state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the owner to set.
    pub fn add_owner(&mut self, name: usize, counter: usize) {
        self.owner_name.push(OwnerName {
            name: name,
            counter: counter,
        });
    }

    pub fn get_owner(&self, lookup: &FxHashMap<usize, String>) -> String {
        self.owner_name
            .iter()
            .map(|e: &OwnerName| lookup[&e.name].clone())
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn set_template_id(&mut self, name: usize) {
        self.template_id = name;
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
    pub fn set_symval(&mut self, name: SymbolicName, value: SymbolicValue) {
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
    pub fn get_symval(&self, name: &SymbolicName) -> Option<&Box<SymbolicValue>> {
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
    pub template_parameter_names: Vec<usize>,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub unrolled_outputs: HashSet<SymbolicName>,
    pub var2type: FxHashMap<usize, VariableType>,
    pub body: Vec<DebugStatement>,
}

/// Represents a symbolic function used in the symbolic execution process.
#[derive(Default, Clone, Debug)]
pub struct SymbolicFunction {
    pub function_argument_names: Vec<usize>,
    pub body: Vec<DebugStatement>,
}

/// Represents a symbolic component used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicComponent {
    pub template_name: usize,
    pub args: Vec<Box<SymbolicValue>>,
    pub inputs: FxHashMap<usize, Option<SymbolicValue>>,
    pub is_done: bool,
}

// Registers library template by extracting input signals from block statement body provided along with template parameter names list.
//
// # Arguments
//
// * 'name' : Name under which template will be registered within library .
// * 'body' : Block statement serving as main logic body defining behavior captured by template .
// * 'template_parameter_names': List containing names identifying parameters used within template logic .
pub fn register_library(
    template_library: &mut FxHashMap<usize, Box<SymbolicTemplate>>,
    name2id: &mut FxHashMap<String, usize>,
    id2name: &mut FxHashMap<usize, String>,
    name: String,
    body: &Statement,
    template_parameter_names: &Vec<String>,
) {
    let mut inputs: Vec<usize> = vec![];
    let mut outputs: Vec<usize> = vec![];
    let mut var2type: FxHashMap<usize, VariableType> = FxHashMap::default();

    let i = if let Some(i) = name2id.get(&name) {
        *i
    } else {
        name2id.insert(name.clone(), name2id.len());
        id2name.insert(name2id[&name], name);
        name2id.len() - 1
    };

    let dbody = DebugStatement::from(body.clone(), name2id, id2name);
    match dbody {
        DebugStatement::Block { ref stmts, .. } => {
            for s in stmts {
                if let DebugStatement::InitializationBlock {
                    initializations, ..
                } = &s
                {
                    for init in initializations {
                        if let DebugStatement::Declaration { name, xtype, .. } = &init {
                            var2type.insert(name.clone(), xtype.clone());
                            if let VariableType::Signal(typ, _taglist) = &xtype {
                                match typ {
                                    SignalType::Input => {
                                        inputs.push(*name);
                                    }
                                    SignalType::Output => {
                                        outputs.push(*name);
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
        template_parameter_names: template_parameter_names
            .iter()
            .map(|p: &String| name2id[p])
            .collect::<Vec<_>>(),
        inputs: inputs,
        outputs: outputs,
        unrolled_outputs: HashSet::new(),
        var2type: var2type,
        body: vec![dbody.clone(), DebugStatement::Ret],
    };
    template_library.insert(i, Box::new(template));
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
pub struct SymbolicExecutor<'a> {
    pub template_library: Box<FxHashMap<usize, Box<SymbolicTemplate>>>,
    pub name2id: &'a mut FxHashMap<String, usize>,
    pub id2name: &'a mut FxHashMap<usize, String>,
    pub function_library: FxHashMap<usize, Box<SymbolicFunction>>,
    pub function_counter: FxHashMap<usize, usize>,
    pub components_store: FxHashMap<SymbolicName, SymbolicComponent>,
    pub variable_types: FxHashMap<usize, DebugVariableType>,
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

impl<'a> SymbolicExecutor<'a> {
    /// Creates a new instance of `SymbolicExecutor`, initializing all necessary states and statistics trackers.
    pub fn new(
        template_library: Box<FxHashMap<usize, Box<SymbolicTemplate>>>,
        name2id: &'a mut FxHashMap<String, usize>,
        id2name: &'a mut FxHashMap<usize, String>,
        propagate_substitution: bool,
        prime: BigInt,
    ) -> Self {
        SymbolicExecutor {
            template_library: template_library,
            name2id,
            id2name,
            function_library: FxHashMap::default(),
            function_counter: FxHashMap::default(),
            components_store: FxHashMap::default(),
            variable_types: FxHashMap::default(),
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
        self.components_store.clear();
        self.cur_state = SymbolicState::new();
        self.block_end_states.clear();
        self.final_states.clear();
        self.max_depth = 0;

        for (k, _) in self.function_library.iter() {
            self.function_counter.insert(*k, 0_usize);
        }
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
    fn is_ready(&self, name: SymbolicName) -> bool {
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
        let mut name2id = self.name2id.clone();
        let mut id2name = self.id2name.clone();
        for (n, a) in names.iter().zip(args.iter()) {
            let evaled_a = self.evaluate_expression(&DebugExpression::from(
                a.clone(),
                &mut name2id,
                &mut id2name,
            ));
            self.cur_state.set_symval(
                SymbolicName {
                    name: name2id[n],
                    owner: self.cur_state.owner_name.clone(),
                    access: Vec::new(),
                },
                evaled_a,
            );
        }
        //self.name2id = cloned_name2id;
        //self.id2name = cloned_id2name;
    }

    pub fn register_function(
        &mut self,
        name: String,
        body: Statement,
        function_argument_names: &Vec<String>,
    ) {
        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name);
            self.name2id.len() - 1
        };

        let dbody = DebugStatement::from(body, &mut self.name2id, &mut self.id2name);
        self.function_library.insert(
            i,
            Box::new(SymbolicFunction {
                function_argument_names: function_argument_names
                    .iter()
                    .map(|p: &String| self.name2id[p])
                    .collect::<Vec<_>>(),
                body: vec![dbody, DebugStatement::Ret],
            }),
        );
        self.function_counter.insert(i, 0_usize);
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
        statements: &Vec<DebugStatement>,
        cur_bid: usize,
        depth: usize,
    ) {
        let drained_states: Vec<_> = self.block_end_states.drain(..).collect();
        for state in drained_states {
            self.cur_state = *state;
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
    pub fn execute(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.max_depth = max(self.max_depth, self.cur_state.get_depth());

            /*
            let n = SymbolicName {
                name: self.name2id["i"],
                owner: vec![OwnerName {
                    name: self.name2id["main"],
                    counter: 0,
                }],
                access: Vec::new(),
            };
            if self.cur_state.values.contains_key(&n) {
                println!(
                    "main.i={}",
                    self.cur_state.values[&n].lookup_fmt(&self.id2name)
                );
            }*/

            match &statements[cur_bid] {
                DebugStatement::InitializationBlock {
                    initializations,
                    xtype,
                    ..
                } => {
                    let mut is_input = false;
                    if let VariableType::Signal(SignalType::Input, _taglist) = &xtype {
                        is_input = true;
                    }

                    if !(self.skip_initialization_blocks && is_input) {
                        for init in initializations {
                            self.execute(&vec![init.clone()], 0);
                        }
                    }
                    self.block_end_states = vec![Box::new(self.cur_state.clone())];
                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                }
                DebugStatement::Block { meta, stmts, .. } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    self.execute(&stmts, 0);
                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                }
                DebugStatement::IfThenElse {
                    meta,
                    cond,
                    if_case,
                    else_case,
                    ..
                } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    let tmp_cond = self.evaluate_expression(cond);
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
                                format!("(elem_id={}) 🚧 Unreachable `Then` Branch", meta.elem_id)
                                    .yellow()
                            );
                        }
                    } else {
                        if_state.push_trace_constraint(&evaled_condition);
                        if_state.push_side_constraint(&original_evaled_condition);
                        if_state.set_depth(cur_depth + 1);
                        self.cur_state = if_state.clone();
                        self.execute(&vec![*if_case.clone()], 0);
                        self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                    }

                    let mut stack_states_if_true = self.block_end_states.clone();
                    self.block_end_states = stack_states;
                    let neg_evaled_condition =
                        if let SymbolicValue::ConstantBool(v) = evaled_condition {
                            SymbolicValue::ConstantBool(!v)
                        } else {
                            SymbolicValue::UnaryOp(
                                DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                Box::new(evaled_condition),
                            )
                        };
                    let original_neg_evaled_condition =
                        if let SymbolicValue::ConstantBool(v) = original_evaled_condition {
                            SymbolicValue::ConstantBool(!v)
                        } else {
                            SymbolicValue::UnaryOp(
                                DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                Box::new(original_evaled_condition),
                            )
                        };
                    if let SymbolicValue::ConstantBool(false) = neg_evaled_condition {
                        if !self.off_trace {
                            trace!(
                                "{}",
                                format!("(elem_id={}) 🚧 Unreachable `Else` Branch", meta.elem_id)
                                    .yellow()
                            );
                        }
                    } else {
                        else_state.push_trace_constraint(&neg_evaled_condition);
                        else_state.push_side_constraint(&original_neg_evaled_condition);
                        else_state.set_depth(cur_depth + 1);
                        self.cur_state = else_state;
                        if let Some(else_stmt) = else_case {
                            self.execute(&vec![*else_stmt.clone()], 0);
                        } else {
                            self.block_end_states = vec![Box::new(self.cur_state.clone())];
                        }
                        self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                    }
                    self.block_end_states.append(&mut stack_states_if_true);
                }
                DebugStatement::While {
                    meta, cond, stmt, ..
                } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    // Symbolic execution of loops is complex. This is a simplified approach.
                    let tmp_cond = self.evaluate_expression(cond);
                    let evaled_condition =
                        self.fold_variables(&tmp_cond, !self.propagate_substitution);

                    if let SymbolicValue::ConstantBool(flag) = evaled_condition {
                        if flag {
                            self.execute(&vec![*stmt.clone()], 0);
                            self.block_end_states.pop();
                            self.execute(statements, cur_bid);
                        } else {
                            self.block_end_states.push(Box::new(self.cur_state.clone()));
                        }
                    } else {
                        panic!("This tool currently cannot handle the symbolic condition of While Loop: {}", evaled_condition.lookup_fmt(&self.id2name));
                    }

                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                    // Note: This doesn't handle loop invariants or fixed-point computation
                }
                DebugStatement::Return { meta, value, .. } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    let tmp_val = self.evaluate_expression(value);
                    let return_value = self.fold_variables(&tmp_val, !self.propagate_substitution);
                    // Handle return value (e.g., store in a special "return" variable)

                    if !self.id2name.contains_key(&usize::MAX) {
                        self.name2id.insert("__return__".to_string(), usize::MAX);
                        self.id2name.insert(usize::MAX, "__return__".to_string());
                    }

                    self.cur_state.set_symval(
                        SymbolicName {
                            name: usize::MAX,
                            owner: self.cur_state.owner_name.clone(),
                            access: Vec::new(),
                        },
                        return_value,
                    );
                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::Declaration {
                    name,
                    dimensions,
                    xtype,
                    ..
                } => {
                    /*
                    let var_name = if dimensions.is_empty() {
                        format!("{}.{}", self.cur_state.get_owner(), name.clone())
                    } else {
                        //"todo".to_string()
                        format!("{}.{}<{}>", self.cur_state.get_owner(), name, &dimensions)
                    };*/
                    let var_name = SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: Vec::new(),
                    };
                    self.variable_types
                        .insert(*name, DebugVariableType(xtype.clone()));
                    let value = SymbolicValue::Variable(var_name.clone());
                    self.cur_state.set_symval(var_name, value);
                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::Substitution {
                    meta,
                    var,
                    access,
                    op,
                    rhe,
                } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    let expr = self.evaluate_expression(rhe);
                    let original_value = self.fold_variables(&expr, true);
                    let value = self.fold_variables(&expr, !self.propagate_substitution);

                    /*
                    let accessed_name = if access.is_empty() {
                        var.clone()
                    } else {
                        format!(
                            "{}{}",
                            var,
                            &access
                                .iter()
                                .map(|arg0: &DebugAccess| self.evaluate_access(&arg0.clone(),))
                                .map(|debug_access| debug_access.to_string())
                                .collect::<Vec<_>>()
                                .join("")
                        )
                    };
                    */
                    //let var_name =
                    //    format!("{}.{}", self.cur_state.get_owner(), accessed_name.clone());

                    let var_name = SymbolicName {
                        name: *var,
                        owner: self.cur_state.owner_name.clone(),
                        access: access
                            .iter()
                            .map(|arg0: &DebugAccess| self.evaluate_access(&arg0.clone()))
                            .collect::<Vec<_>>(),
                    };

                    if self.keep_track_unrolled_offset {
                        if self
                            .template_library
                            .contains_key(&self.cur_state.template_id)
                            && self.template_library[&self.cur_state.template_id]
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
                            if let DebugAccess::ComponentAccess(tmp_name) = acc {
                                if let Some(component) = self.components_store.get_mut(&var_name) {
                                    component
                                        .inputs
                                        .insert(tmp_name.clone(), Some(value.clone()));
                                }
                            }
                        }

                        if self.is_ready(var_name.clone()) {
                            if !self.components_store[&var_name].is_done {
                                let name2id = &mut self.name2id;
                                let id2name = &mut self.id2name;

                                let mut subse = SymbolicExecutor::new(
                                    self.template_library.clone(),
                                    name2id,
                                    id2name,
                                    self.propagate_substitution,
                                    self.prime.clone(),
                                );

                                //subse.template_library = self.template_library.clone();
                                subse.function_library = self.function_library.clone();
                                subse.function_counter = self.function_counter.clone();
                                subse.cur_state.owner_name = self.cur_state.owner_name.clone();
                                subse.cur_state.add_owner(*var, 0);

                                let templ = &self.template_library
                                    [&self.components_store[&var_name].template_name];
                                subse.cur_state.set_template_id(
                                    self.components_store[&var_name].template_name.clone(),
                                );

                                for i in 0..(templ.template_parameter_names.len()) {
                                    let n = SymbolicName {
                                        name: templ.template_parameter_names[i],
                                        owner: subse.cur_state.owner_name.clone(),
                                        access: Vec::new(),
                                    };
                                    subse.cur_state.set_symval(
                                        n,
                                        *self.components_store[&var_name].args[i].clone(),
                                    );
                                }

                                for (k, v) in self.components_store[&var_name].inputs.iter() {
                                    let n = SymbolicName {
                                        name: *k,
                                        owner: subse.cur_state.owner_name.clone(),
                                        access: Vec::new(),
                                    };
                                    subse.cur_state.set_symval(n, v.clone().unwrap());
                                }

                                if !self.off_trace {
                                    trace!(
                                        "{}",
                                        format!("{}", "===========================").cyan()
                                    );
                                    trace!(
                                        "📞 Call {}",
                                        subse.id2name
                                            [&self.components_store[&var_name].template_name]
                                    );
                                }

                                subse.execute(&templ.body, 0);

                                if subse.final_states.len() > 1 {
                                    warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                                }
                                self.cur_state
                                    .trace_constraints
                                    .append(&mut subse.final_states[0].trace_constraints);
                                self.cur_state
                                    .side_constraints
                                    .append(&mut subse.final_states[0].side_constraints);
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
                            if self.template_library.contains_key(&callee_name) {
                                let mut comp_inputs: FxHashMap<usize, Option<SymbolicValue>> =
                                    FxHashMap::default();
                                for inp_name in &self.template_library[&callee_name].inputs.clone()
                                {
                                    comp_inputs.insert(inp_name.clone(), None);
                                }
                                let c = SymbolicComponent {
                                    template_name: callee_name.clone(),
                                    args: args.clone(),
                                    inputs: comp_inputs,
                                    is_done: false,
                                };
                                self.components_store.insert(var_name.clone(), c);
                            }
                        }
                        _ => {
                            if self.variable_types[var].0 != VariableType::Var {
                                let cont = SymbolicValue::BinaryOp(
                                    Box::new(SymbolicValue::Variable(var_name.clone())),
                                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                    Box::new(value),
                                );
                                self.cur_state.push_trace_constraint(&cont);

                                if let DebugAssignOp(AssignOp::AssignConstraintSignal) = op {
                                    let original_cont = SymbolicValue::BinaryOp(
                                        Box::new(SymbolicValue::Variable(var_name)),
                                        DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                        Box::new(original_value),
                                    );
                                    self.cur_state.push_side_constraint(&original_cont);
                                }
                            }
                        }
                    }

                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::MultSubstitution {
                    meta, lhe, op, rhe, ..
                } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }

                    let lhe_val = self.evaluate_expression(lhe);
                    let rhe_val = self.evaluate_expression(rhe);
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
                    if let DebugAssignOp(AssignOp::AssignConstraintSignal) = op {
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
                DebugStatement::ConstraintEquality { meta, lhe, rhe } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }

                    let lhe_val = self.evaluate_expression(lhe);
                    let rhe_val = self.evaluate_expression(rhe);
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
                DebugStatement::Assert { meta, arg, .. } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    let expr = self.evaluate_expression(&arg);
                    let condition = self.fold_variables(&expr, !self.propagate_substitution);
                    self.cur_state.push_trace_constraint(&condition);
                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: _,
                    rhe: _,
                    ..
                } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    // Underscore substitution doesn't affect the symbolic state
                }
                DebugStatement::LogCall { meta, args: _, .. } => {
                    if !self.off_trace {
                        trace!(
                            "(elem_id={}) {}",
                            meta.elem_id,
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
                    }
                    // Logging doesn't affect the symbolic state
                }
                DebugStatement::Ret => {
                    if !self.off_trace {
                        trace!(
                            "{} {}",
                            format!("{}", "🔙 Ret:").red(),
                            self.cur_state.lookup_fmt(&self.id2name)
                        );
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
        assignment: &FxHashMap<SymbolicName, BigInt>,
        off_trace: bool,
    ) {
        self.cur_state.template_id = self.name2id[id];
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
        self.execute(
            &self.template_library[&self.cur_state.template_id]
                .body
                .clone(),
            0,
        );
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
    fn evaluate_access(&mut self, access: &DebugAccess) -> SymbolicAccess {
        match &access {
            DebugAccess::ComponentAccess(name) => SymbolicAccess::ComponentAccess(name.clone()),
            DebugAccess::ArrayAccess(expr) => {
                let tmp_e = self.evaluate_expression(&expr);
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
            SymbolicValue::Variable(sname) => {
                if only_constatant_folding {
                    if let Some(template) = self.template_library.get(&self.cur_state.template_id) {
                        if let Some(typ) = template.var2type.get(&sname.name) {
                            if let VariableType::Signal(SignalType::Output, _) = typ {
                                return symval.clone();
                            } else if let VariableType::Var = typ {
                                return *self.cur_state.get_symval(&sname).cloned().unwrap_or_else(
                                    || Box::new(SymbolicValue::Variable(sname.clone())),
                                );
                            }
                        }
                    }
                    if let Some(boxed_value) = self.cur_state.get_symval(&sname) {
                        if let SymbolicValue::ConstantInt(v) = *boxed_value.clone() {
                            return SymbolicValue::ConstantInt(v);
                        }
                    }
                    symval.clone()
                } else {
                    *self
                        .cur_state
                        .get_symval(&sname)
                        .cloned()
                        .unwrap_or_else(|| Box::new(SymbolicValue::Variable(sname.clone())))
                }
            }
            SymbolicValue::BinaryOp(lv, infix_op, rv) => {
                let lhs = self.fold_variables(lv, only_constatant_folding);
                let rhs = self.fold_variables(rv, only_constatant_folding);
                match (&lhs, &rhs) {
                    (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                        match &infix_op.0 {
                            ExpressionInfixOpcode::Add => {
                                SymbolicValue::ConstantInt((lv + rv) % &self.prime)
                            }
                            ExpressionInfixOpcode::Sub => {
                                SymbolicValue::ConstantInt((lv - rv) % &self.prime)
                            }
                            ExpressionInfixOpcode::Mul => {
                                SymbolicValue::ConstantInt((lv * rv) % &self.prime)
                            }
                            ExpressionInfixOpcode::Div => {
                                if rv.is_zero() {
                                    SymbolicValue::ConstantInt(BigInt::zero())
                                } else {
                                    let mut r = self.prime.clone();
                                    let mut new_r = rv.clone();
                                    if r.is_negative() {
                                        r += &self.prime;
                                    }
                                    if new_r.is_negative() {
                                        new_r += &self.prime;
                                    }

                                    let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
                                    rv_inv %= self.prime.clone();
                                    if rv_inv.is_negative() {
                                        rv_inv += &self.prime;
                                    }

                                    SymbolicValue::ConstantInt((lv * rv_inv) % &self.prime)
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
                                SymbolicValue::ConstantBool(lv % &self.prime < rv % &self.prime)
                            }
                            ExpressionInfixOpcode::Greater => {
                                SymbolicValue::ConstantBool(lv % &self.prime > rv % &self.prime)
                            }
                            ExpressionInfixOpcode::LesserEq => {
                                SymbolicValue::ConstantBool(lv % &self.prime <= rv % &self.prime)
                            }
                            ExpressionInfixOpcode::GreaterEq => {
                                SymbolicValue::ConstantBool(lv % &self.prime >= rv % &self.prime)
                            }
                            ExpressionInfixOpcode::Eq => {
                                SymbolicValue::ConstantBool(lv % &self.prime == rv % &self.prime)
                            }
                            ExpressionInfixOpcode::NotEq => {
                                SymbolicValue::ConstantBool(lv % &self.prime != rv % &self.prime)
                            }
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
                    .map(|e| Box::new(self.fold_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::Tuple(elements) => SymbolicValue::Tuple(
                elements
                    .iter()
                    .map(|e| Box::new(self.fold_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => SymbolicValue::UniformArray(
                Box::new(self.fold_variables(element, only_constatant_folding)),
                Box::new(self.fold_variables(count, only_constatant_folding)),
            ),
            SymbolicValue::Call(func_name, args) => SymbolicValue::Call(
                func_name.clone(),
                args.iter()
                    .map(|arg| Box::new(self.fold_variables(arg, only_constatant_folding)))
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
        match &expr {
            DebugExpression::Number(_meta, value) => SymbolicValue::ConstantInt(value.clone()),
            DebugExpression::Variable {
                name,
                access,
                meta: _,
            } => {
                let resolved_name = if access.is_empty() {
                    SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: Vec::new(),
                    }
                } else {
                    let tmp_name = SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: Vec::new(),
                    };
                    let sv = self.cur_state.get_symval(&tmp_name).cloned();
                    let evaluated_access = access
                        .iter()
                        .map(|arg0: &DebugAccess| self.evaluate_access(&arg0))
                        .collect::<Vec<_>>();

                    if evaluated_access.len() == 1 && sv.is_some() {
                        if let SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(a)) =
                            &evaluated_access[0]
                        {
                            match *sv.unwrap().clone() {
                                SymbolicValue::Array(values) => {
                                    return *values[a.to_usize().unwrap()].clone();
                                }
                                _ => {}
                            }
                        }
                    }

                    SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: evaluated_access,
                    }
                };
                SymbolicValue::Variable(resolved_name)
            }
            DebugExpression::InfixOp {
                meta: _,
                lhe,
                infix_op,
                rhe,
            } => {
                let lhs = self.evaluate_expression(lhe);
                let rhs = self.evaluate_expression(rhe);
                SymbolicValue::BinaryOp(Box::new(lhs), infix_op.clone(), Box::new(rhs))
            }
            DebugExpression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => {
                let expr = self.evaluate_expression(rhe);
                SymbolicValue::UnaryOp(prefix_op.clone(), Box::new(expr))
            }
            DebugExpression::InlineSwitchOp {
                meta: _,
                cond,
                if_true,
                if_false,
            } => {
                let condition = self.evaluate_expression(cond);
                let true_branch = self.evaluate_expression(if_true);
                let false_branch = self.evaluate_expression(if_false);
                SymbolicValue::Conditional(
                    Box::new(condition),
                    Box::new(true_branch),
                    Box::new(false_branch),
                )
            }
            DebugExpression::ParallelOp { rhe, .. } => self.evaluate_expression(rhe),
            DebugExpression::ArrayInLine { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| Box::new(self.evaluate_expression(v)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebugExpression::Tuple { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| Box::new(self.evaluate_expression(v)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebugExpression::UniformArray {
                value, dimension, ..
            } => {
                let evaluated_value = self.evaluate_expression(value);
                let evaluated_dimension = self.evaluate_expression(dimension);
                SymbolicValue::UniformArray(
                    Box::new(evaluated_value),
                    Box::new(evaluated_dimension),
                )
            }
            DebugExpression::Call { id, args, .. } => {
                let tmp_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect();
                let evaluated_args = tmp_args
                    .iter()
                    .map(|arg| Box::new(self.fold_variables(&arg, false)))
                    .collect();
                if self.template_library.contains_key(id) {
                    SymbolicValue::Call(id.clone(), evaluated_args)
                } else if self.function_library.contains_key(id) {
                    let name2id = &mut self.name2id;
                    let id2name = &mut self.id2name;
                    let mut subse = SymbolicExecutor::new(
                        self.template_library.clone(),
                        name2id,
                        id2name,
                        self.propagate_substitution,
                        self.prime.clone(),
                    );
                    subse.cur_state.owner_name = self.cur_state.owner_name.clone();
                    subse.cur_state.add_owner(*id, self.function_counter[id]);
                    //subse.template_library = self.template_library.clone();
                    subse.function_library = self.function_library.clone();
                    self.function_counter
                        .insert(*id, self.function_counter[id] + 1);
                    subse.function_counter = self.function_counter.clone();

                    let func = &self.function_library[id];
                    for i in 0..(func.function_argument_names.len()) {
                        let sname = SymbolicName {
                            name: func.function_argument_names[i],
                            owner: subse.cur_state.owner_name.clone(),
                            access: Vec::new(),
                        };
                        subse
                            .cur_state
                            .set_symval(sname, *evaluated_args[i].clone());
                    }

                    if !self.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                        trace!("📞 Call {}", subse.id2name[id]);
                    }

                    subse.execute(&func.body, 0);

                    if subse.final_states.len() > 1 {
                        warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                    }
                    //let mut sub_trace_constraints = subse.final_states[0].trace_constraints.clone();
                    //let mut sub_side_constraints = subse.final_states[0].side_constraints.clone();
                    self.cur_state
                        .trace_constraints
                        .append(&mut subse.final_states[0].trace_constraints);
                    self.cur_state
                        .side_constraints
                        .append(&mut subse.final_states[0].side_constraints);
                    //self.name2id = subse.name2id;
                    //self.id2name = subse.id2name;

                    if !self.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    self.function_counter = subse.function_counter.clone();

                    let sname = SymbolicName {
                        name: usize::MAX,
                        owner: subse.final_states[0].owner_name.clone(),
                        access: Vec::new(),
                    };

                    *subse.final_states[0].values[&sname].clone()
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
                panic!("Unhandled expression type: {:?}", expr);
                //SymbolicValue::Variable(format!("Unhandled({:?})", expr), "".to_string())
            }
        }
    }
}
