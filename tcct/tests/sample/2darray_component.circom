pragma circom 2.0.0;

template Callee() {
    signal input x[2][2];
    signal output y[2];
    y[0] <-- (x[0][0] + x[0][1]) / x[0][0];
    y[0] * x[0][0] === x[0][0] + x[0][1];
    y[1] <== (x[1][0] + x[1][1]) / x[1][1];
    y[1] * x[1][1] === x[1][0] + x[1][1];
}

template Caller() {
    signal input in[4];
    signal output out;
    
    component c = Callee();
    c.x[0][0] <== in[0];
    c.x[0][1] <== in[1];
    c.x[1][0] <== in[2];
    c.x[1][1] <== in[3];
    out <== c.y[0] + c.y[1];
}

component main = Caller();
