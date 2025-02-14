pragma circom 2.0.0;

template Callee() {
    signal input x[2];
    signal output y[2];
    y[0] <-- x[0] / x[1];
    y[0] * x[1] === x[0];
    y[1] <== x[1] / x[0];
    y[1] * x[0] === x[1];
}

template Caller(N) {
    signal input a;
    signal input b;
    signal output out[N];
    
    component c[N];

    var i;
    for (i=0; i < N; i++) {
        c[i] = Callee();
        c[i].x[0] <== a;
        c[i].x[1] <== b;
        out[i] <== c[i].y[0] + c[i].y[1];
    }
}

component main = Caller(2);
