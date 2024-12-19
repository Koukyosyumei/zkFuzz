pragma circom 2.0.0;

template Callee(N) {
    signal input in[2][N];
    signal output out;

    var i;
    var sum;
    for (i = 0; i < N; i++) {
        sum += in[0][i];
    }
    out <== sum;
}

template Caller(N) {
    signal input in[1][2][N];
    signal output out;
    component c = Callee(N);
    c.in <== in[1];
    out <== c.out;
}

component main = Caller(3);
