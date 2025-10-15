pragma circom 2.1.6;

include "../circomlib-cff5ab6/poseidon.circom";

template Example () {
    signal input a;
    signal input b;
    
    component hash = Poseidon(1);
    hash.inputs[0] <== a;

    log("hash", hash.out);
    assert(b==hash.out);
}