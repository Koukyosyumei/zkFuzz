pragma circom 2.0.0;

template SingleAssignment0() {
    signal input a;
    signal input b;
    signal output out;
    out <-- a + 1;
    out === b + 1;
}

component main = SingleAssignment0();