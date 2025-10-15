pragma circom 2.0.0;

include "./mux.circom";
include "./compare.circom";

template PushSuppl(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal input sp;
    signal input operand;
    signal output stack_next[STACK_SIZE];

    component cmp_push[STACK_SIZE];
    signal is_index_push[STACK_SIZE];
    signal top_vl;
    signal tmp0[STACK_SIZE];
    signal tmp1[STACK_SIZE];

    for (var i = 0; i < STACK_SIZE; i++) {
        cmp_push[i] = IsEqual();
        cmp_push[i].in[0] <== i;
        cmp_push[i].in[1] <== sp + 1;
        is_index_push[i] <== cmp_push[i].out;
        tmp0[i] <== is_index_push[i] * operand;
        tmp1[i] <== (1 - is_index_push[i]) * stack[i];
        stack_next[i] <== tmp0[i] + tmp1[i];
    }
}

template DropSuppl(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal output stack_next[STACK_SIZE];

    for (var i = 1; i < STACK_SIZE; i++) {
        stack_next[i - 1] <== stack[i];
    }
    stack_next[STACK_SIZE - 1] <== 0;
}

template AddSuppl(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal input sp;
    signal output stack_next[STACK_SIZE];

    component cmp_add[STACK_SIZE];
    signal is_index_add[STACK_SIZE];
    signal tmp0[STACK_SIZE];
    signal tmp1[STACK_SIZE];

    for  (var i = 0; i < STACK_SIZE - 1; i++) {
        cmp_add[i] = IsEqual();
        cmp_add[i].in[0] <== i;
        cmp_add[i].in[1] <== sp - 1;
        is_index_add[i] <== cmp_add[i].out;
        tmp0[i] <== is_index_add[i] * (stack[i] + stack[i + 1]);
        tmp1[i] <== (1 - is_index_add[i]) * stack[i];
        stack_next[i] <== tmp0[i] + tmp1[i];
    }
    stack_next[STACK_SIZE - 1] <== stack[STACK_SIZE - 1];
}

template SubSuppl(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal input sp;
    signal output stack_next[STACK_SIZE];

    component cmp_sub[STACK_SIZE];
    signal is_index_sub[STACK_SIZE];
    signal tmp0[STACK_SIZE];
    signal tmp1[STACK_SIZE];

    for  (var i = 0; i < STACK_SIZE - 1; i++) {
        cmp_sub[i] = IsEqual();
        cmp_sub[i].in[0] <== i;
        cmp_sub[i].in[1] <== sp - 1;
        is_index_sub[i] <== cmp_sub[i].out;
        tmp0[i] <== is_index_sub[i] * (stack[i + 1] - stack[i]);
        tmp1[i] <== (1 - is_index_sub[i]) * stack[i];
        stack_next[i] <== tmp0[i] + tmp1[i];
    }
    stack_next[STACK_SIZE - 1] <== stack[STACK_SIZE - 1];
}

template PickSuppl(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal input sp;
    signal input operand;
    signal input opcode;
    signal output stack_next[STACK_SIZE];

    component mux_pick = SinglMux(STACK_SIZE);
    component cmp_op = IsEqual();
    component cmp_pick[STACK_SIZE];
    signal tv; // target value
    signal is_index_pick[STACK_SIZE];
    signal tmp0[STACK_SIZE];
    signal tmp1[STACK_SIZE];

    cmp_op.in[0] <== opcode;
    cmp_op.in[1] <== 6;
    signal is_pick <== cmp_op.out;

    var tp = (sp - operand) * is_pick; // target pointer
    mux_pick.inp <== stack;
    mux_pick.sel <== tp;
    tv <== mux_pick.out;

    for (var i = 0; i < STACK_SIZE; i++) {
        cmp_pick[i] = IsEqual();
        cmp_pick[i].in[0] <== i;
        cmp_pick[i].in[1] <== sp + 1;
        is_index_pick[i] <== cmp_pick[i].out;
        tmp0[i] <== is_index_pick[i] * tv;
        tmp1[i] <== (1 - is_index_pick[i]) * stack[i];
        stack_next[i] <== tmp0[i] + tmp1[i];
    }
}


template NextStack(STACK_SIZE) {
    signal input stack[STACK_SIZE];
    signal input sp;
    signal input opcode;
    signal input operand;
    signal output stack_next[STACK_SIZE];

    signal stack_candidates[9][STACK_SIZE];

    component mux = MultiMux(9, STACK_SIZE);

    // 0. HALT (do nothing)
    stack_candidates[0] <== stack;

    // 1. PUSH
    component push_suppl = PushSuppl(STACK_SIZE);
    push_suppl.stack <== stack;
    push_suppl.sp <== sp;
    push_suppl.operand <== operand;
    stack_candidates[1] <== push_suppl.stack_next;

    // 2. POP: do nothing because sp is already decremented in NextSP
    stack_candidates[2] <== stack;

    // 3. DROP
    component drop_suppl = DropSuppl(STACK_SIZE);
    drop_suppl.stack <== stack;
    stack_candidates[3] <== drop_suppl.stack_next;

    // 4. ADD
    component add_suppl = AddSuppl(STACK_SIZE);
    add_suppl.stack <== stack;
    add_suppl.sp <== sp;
    stack_candidates[4] <== add_suppl.stack_next;

    // 5. SUB
    component sub_suppl = SubSuppl(STACK_SIZE);
    sub_suppl.stack <== stack;
    sub_suppl.sp <== sp;
    stack_candidates[5] <== sub_suppl.stack_next;

    // 6. PICK
    component pick_suppl = PickSuppl(STACK_SIZE);
    pick_suppl.stack <== stack;
    pick_suppl.sp <== sp;
    pick_suppl.operand <== operand;
    pick_suppl.opcode <== opcode;
    stack_candidates[6] <== pick_suppl.stack_next;

    // 7. JMP (do nothing)
    stack_candidates[7] <== stack;

    // 8. JZ (do nothing)
    stack_candidates[8] <== stack;

    mux.inp <== stack_candidates;
    mux.sel <== opcode;
    stack_next <== mux.out;
}


