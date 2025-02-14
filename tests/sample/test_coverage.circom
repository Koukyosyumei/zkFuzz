pragma circom 2.0.0;

template Main(N) {
    signal input in[N];
    signal output out;

    var i;
    var sum = 0;
    for (i=0; i < N; i++) {
        sum += in[i] > 0 ? 1 : 0;
    }

    out <-- sum;
}

component main = Main(2);
