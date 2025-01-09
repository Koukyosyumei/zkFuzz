pragma circom 2.0.0;

template Callee(P) {
    signal input in[2];
    signal output out;
    out <== P * (in[0] + in[1]);
}

template Caller() {
    signal input a;
    signal input b;
    signal output c;
    c <== Callee(3)([a, b]);
}

component main = Caller();

