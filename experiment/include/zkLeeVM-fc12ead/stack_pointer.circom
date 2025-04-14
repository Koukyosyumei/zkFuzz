pragma circom 2.0.0;

include "./mux.circom";

template NextSP() {
    var INSTR_SET_SIZE = 9;
    signal input sp;
    signal input opcode;
    signal output sp_next;

    signal sp_candidates[INSTR_SET_SIZE];

    component mux = SinglMux(INSTR_SET_SIZE);

    sp_candidates[0] <== sp;     // HALT
    sp_candidates[1] <== sp + 1; // PUSH
    sp_candidates[2] <== sp - 1; // POP
    sp_candidates[3] <== sp - 1; // DROP
    sp_candidates[4] <== sp - 1; // ADD
    sp_candidates[5] <== sp - 1; // SUB
    sp_candidates[6] <== sp + 1; // PICK
    sp_candidates[7] <== sp;     // JMP
    sp_candidates[8] <== sp - 1; // JZ

    mux.sel <== opcode;
    mux.inp <== sp_candidates;
    sp_next <== mux.out;
}
