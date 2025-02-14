use std::rc::Rc;

use colored::Colorize;
use rustc_hash::FxHashMap;

use crate::executor::symbolic_value::{
    OwnerName, SymbolicAccess, SymbolicName, SymbolicValue, SymbolicValueRef,
};
use crate::executor::utils::italic;

pub type SymbolBindingMap = FxHashMap<SymbolicName, SymbolicValueRef>;
pub type SymbolicTrace = Vec<SymbolicValueRef>;
pub type SymbolicConstraints = Vec<SymbolicValueRef>;

/// Represents the state of symbolic execution, holding symbolic values,
/// trace constraints, side constraints, and depth information.
#[derive(Clone)]
pub struct SymbolicState {
    pub owner_name: Rc<Vec<OwnerName>>,
    pub template_id: usize,
    pub is_within_initialization_block: bool,
    pub contains_symbolic_loop: bool,
    pub depth: usize,
    pub symbol_binding_map: SymbolBindingMap,
    pub symbolic_trace: SymbolicTrace,
    pub side_constraints: SymbolicConstraints,
    pub is_failed: bool,
}

impl SymbolicState {
    /// Creates a new `SymbolicState` with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `SymbolicState` with empty fields.
    pub fn new() -> Self {
        SymbolicState {
            owner_name: Rc::new(Vec::new()),
            template_id: usize::MAX,
            is_within_initialization_block: false,
            contains_symbolic_loop: false,
            depth: 0_usize,
            symbol_binding_map: SymbolBindingMap::default(),
            symbolic_trace: SymbolicTrace::new(),
            side_constraints: SymbolicConstraints::new(),
            is_failed: false,
        }
    }

    /// Adds an owner to the current symbolic state.
    ///
    /// This method appends a new owner name to the existing list of owners.
    ///
    /// # Arguments
    ///
    /// * `owner_name` - The `OwnerName` to be added.
    pub fn add_owner(&mut self, owner_name: &OwnerName) {
        let updated_owner_list = Rc::make_mut(&mut self.owner_name);
        updated_owner_list.push(owner_name.clone());
    }

    /// Retrieves the full owner name as a string.
    ///
    /// This method joins all owner names in the current state using a dot separator.
    ///
    /// # Arguments
    ///
    /// * `id2name` - A hash map containing mappings from usize to String for name lookups.
    ///
    /// # Returns
    ///
    /// A string representing the full owner name.
    pub fn get_owner(&self, id2name: &FxHashMap<usize, String>) -> String {
        self.owner_name
            .iter()
            .map(|e: &OwnerName| {
                let access_str: String = if let Some(accesses) = &e.access {
                    accesses
                        .iter()
                        .map(|s: &SymbolicAccess| s.lookup_fmt(id2name))
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    "".to_string()
                };
                id2name[&e.id].clone() + &access_str
            })
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Sets the template ID for the current symbolic state.
    ///
    /// # Arguments
    ///
    /// * `id` - The usize value representing the template ID.
    pub fn set_template_id(&mut self, id: usize) {
        self.template_id = id;
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
    /// * `sym_name` - The name of the variable.
    /// * `sym_val` - The symbolic value to associate with the variable.
    pub fn set_sym_val(&mut self, sym_name: SymbolicName, sym_val: SymbolicValue) {
        self.symbol_binding_map.insert(sym_name, Rc::new(sym_val));
    }

    /// Sets a reference-counted symbolic value for a given variable name in the state.
    ///
    /// # Arguments
    ///
    /// * `sym_name` - The name of the variable.
    /// * `sym_val` - The reference-counted symbolic value to associate with the variable.
    pub fn set_rc_sym_val(&mut self, sym_name: SymbolicName, sym_val: SymbolicValueRef) {
        self.symbol_binding_map.insert(sym_name, sym_val);
    }

    /// Retrieves a symbolic value associated with a given variable name.
    ///
    /// # Arguments
    ///
    /// * `sym_name` - The name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// An optional reference to the symbolic value if it exists.
    pub fn get_sym_val(&self, sym_name: &SymbolicName) -> Option<&SymbolicValueRef> {
        self.symbol_binding_map.get(sym_name)
    }

    pub fn get_sym_val_or_make_symvar(&self, sym_name: &SymbolicName) -> SymbolicValue {
        if let Some(sym_val) = self.symbol_binding_map.get(sym_name) {
            (**sym_val).clone()
        } else {
            SymbolicValue::Variable(sym_name.clone())
        }
    }

    /// Adds a trace constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_symbolic_trace(&mut self, constraint: &SymbolicValue) {
        self.symbolic_trace.push(Rc::new(constraint.clone()));
    }

    /// Adds a side constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_side_constraint(&mut self, constraint: &SymbolicValue) {
        self.side_constraints.push(Rc::new(constraint.clone()));
    }

    /// Formats the symbolic state for lookup and display.
    ///
    /// This method creates a string representation of the symbolic state,
    /// including owner, depth, values, trace constraints, and side constraints.
    ///
    /// # Arguments
    ///
    /// * `id2name` - A hash map containing mappings from usize to String for name lookups.
    ///
    /// # Returns
    ///
    /// A formatted string representation of the symbolic state.
    pub fn lookup_fmt(&self, id2name: &FxHashMap<usize, String>) -> String {
        let mut s = "".to_string();
        s += &format!("🛠️ {}", format!("{}", "SymbolicState [\n").cyan());
        s += &format!(
            "  {} {}\n",
            format!("👤 {}", "owner:").cyan(),
            italic(&format!("{:?}", &self.get_owner(id2name))).magenta()
        );
        s += &format!("  📏 {} {}\n", format!("{}", "depth:").cyan(), self.depth);
        s += &format!("  📋 {}\n", format!("{}", "values:").cyan());
        for (k, v) in self.symbol_binding_map.iter() {
            s += &format!(
                "      {}: {}\n",
                k.lookup_fmt(id2name),
                format!("{}", v.lookup_fmt(id2name))
                    .replace("\n", "")
                    .replace("  ", " ")
            );
        }
        s += &format!(
            "  {} {}\n",
            format!("{}", "🪶 symbolic_trace:").cyan(),
            format!(
                "{}",
                &self
                    .symbolic_trace
                    .iter()
                    .map(|c| c.lookup_fmt(id2name))
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
                    .map(|c| c.lookup_fmt(id2name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace("\n", "")
            .replace("  ", " ")
            .replace("  ", " ")
        );
        s += &format!(
            "  {} {}\n",
            format!("{}", "➰ contains_symbolic_loop:").cyan(),
            self.contains_symbolic_loop
        );
        s += &format!("{}\n", format!("{}", "]").cyan());
        s
    }
}
