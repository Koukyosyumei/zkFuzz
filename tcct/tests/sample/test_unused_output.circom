pragma circom 2.0.0;

template UnusedOutput() {
    signal input a;
    signal input b;
    signal output out[2];
    out[0] <== a + b;
}

component main = UnusedOutput();