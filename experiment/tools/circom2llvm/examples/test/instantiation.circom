pragma circom 2.0.0;

template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1=0;

    var e2=1;
    for (var i = 0; i<n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] -1 ) === 0;
        lc1 += out[i] * e2;
        e2 = e2+e2;
    }

    lc1 === in;
}

template Split(n, m) {
    assert(n <= 126);
    signal input in;
    signal output small;
    signal output big;

    small <-- in % (1 << n);
    big <-- in \ (1 << n);

    component n2b_small = Num2Bits(n);
    n2b_small.in <== small;
    component n2b_big = Num2Bits(m);
    n2b_big.in <== big;

    in === small + big * (1 << n);
}

component main{public [in]} = Split(3, 5);
