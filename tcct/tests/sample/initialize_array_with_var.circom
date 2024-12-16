pragma circom 2.0.0;

template Callee() {
    var dim = 2;
    signal input in[dim];
    signal output out;

    var i;
    var sum;
    for (i = 0; i < dim; i++) {
        sum += in[i];
    }
    out <== sum;
}

template Caller() {
    signal input in;
    signal output out;
    
    component c = Callee();
    c.in[0] <== in + 1;
    c.in[1] <== in * 2;
    out <== c.out;
}

component main = Caller();
