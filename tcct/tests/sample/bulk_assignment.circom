pragma circom 2.0.0;

template Callee(N) {
    signal input in[N];
    signal output out;

    var i;
    var sum;
    for (i = 0; i < N; i++) {
        sum += in[i];
    }
    out <== sum;
}

template Caller(N) {
    signal input in[N];
    signal output out;
    component c = Callee(N);
    c.in <== in;
    out <== c.out;
}

component main = Caller(2);
