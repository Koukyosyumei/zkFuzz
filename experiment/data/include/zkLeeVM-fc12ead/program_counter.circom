pragma circom 2.0.0;

include "./mux.circom";
include "./compare.circom";

template NextPC() {
    signal input topvl;
    signal input pc;
    signal input opcode;
    signal input operand;
    signal output pc_next;

    signal pc_candidates[3];

    component mux = SinglMux(3);
    component cmp_halt = IsEqual();
    component cmp_jmp = IsEqual();
    component cmp_jz = IsEqual();
    component cmp_top_zero = IsZero();

    signal is_halt, is_jmp, is_jz, is_top_zero;
    signal is_jz_tz, is_jump;

    cmp_halt.in[0] <== opcode;
    cmp_halt.in[1] <== 0;
    cmp_halt.out ==> is_halt;

    cmp_jmp.in[0] <== opcode;
    cmp_jmp.in[1] <== 7;
    cmp_jmp.out ==> is_jmp;

    cmp_jz.in[0] <== opcode;
    cmp_jz.in[1] <== 8;
    cmp_jz.out ==> is_jz;

    cmp_top_zero.in <== topvl;
    cmp_top_zero.out ==> is_top_zero;

    is_jz_tz <== is_jz * is_top_zero; // is_jz AND is_top_zero
    is_jump <== is_jmp + is_jz_tz - is_jmp * is_jz_tz; // is_jmp OR is_jz_tz

    // If we halt, context is 2
    // If we jump, context is 1
    // Otherwise, context is 0
    var context = is_halt + is_jump * 2;

    pc_candidates[0] <== pc + 1;  // No jump; next instruction
    pc_candidates[1] <== pc;      // Halt: do nothing
    pc_candidates[2] <== operand; // Jump to operand

    mux.sel <== context;
    mux.inp <== pc_candidates;
    pc_next <== mux.out;
}

