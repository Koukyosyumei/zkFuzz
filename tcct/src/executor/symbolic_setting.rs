use num_bigint_dig::BigInt;

#[derive(Clone)]
pub struct SymbolicExecutorSetting {
    pub prime: BigInt,
    pub only_initialization_blocks: bool,
    pub skip_initialization_blocks: bool,
    pub off_trace: bool,
    pub keep_track_constraints: bool,
    pub substitute_output: bool,
    pub propagate_assignments: bool,
}

pub fn get_default_setting_for_symbolic_execution(prime: BigInt) -> SymbolicExecutorSetting {
    SymbolicExecutorSetting {
        prime: prime,
        skip_initialization_blocks: false,
        only_initialization_blocks: false,
        off_trace: false,
        keep_track_constraints: true,
        substitute_output: false,
        propagate_assignments: false,
    }
}

pub fn get_default_setting_for_concrete_execution(prime: BigInt) -> SymbolicExecutorSetting {
    SymbolicExecutorSetting {
        prime: prime,
        skip_initialization_blocks: true,
        only_initialization_blocks: false,
        off_trace: true,
        keep_track_constraints: false,
        substitute_output: true,
        propagate_assignments: true,
    }
}
