use inkwell::{basic_block::BasicBlock, values::InstructionOpcode};

pub fn is_terminated_basicblock(bb: &BasicBlock) -> bool {
    let last_inst_op = bb.get_last_instruction();
    let has_return = match last_inst_op {
        Some(last_inst) => last_inst.get_opcode() == InstructionOpcode::Return,
        None => false,
    };
    return has_return;
}
