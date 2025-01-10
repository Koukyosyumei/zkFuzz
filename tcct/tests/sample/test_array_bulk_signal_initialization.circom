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
    signal input x[2];
    signal output y;
    
    component c = Callee();
    c.in <== x;
    y <== c.out;
}

component main = Caller();
