pragma circom 2.1.6;
include "../circomlib-cff5ab6/poseidon.circom";


template PoseidonCommitment() {
    signal input c_id;
    signal input cr_id;
    signal input t_id;
    signal output out;
    
    component hash = Poseidon(2);
    hash.inputs[0] <== cr_id;
    hash.inputs[1] <== t_id;

    var commitment_assert;
    commitment_assert = c_id == hash.out ? 1 : 0;
    out <-- commitment_assert;
    

    signal dummy;
    dummy <== cr_id * t_id;
}

//component main { public [ c_id, cr_id ] } = PoseidonCommitment();