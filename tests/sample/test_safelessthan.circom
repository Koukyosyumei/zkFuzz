pragma circom 2.0.6;

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

template LessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n+1);

    n2b.in <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.out[n];
}

template VulnerableLessThan() {
    signal input a;
    signal input b;
    signal output out;

    // Assume a and b are 3-bit integers
    component lt = LessThan(3);
    
    // bit-length check
    component n1 = Num2Bits(3);
    n1.in <== a;
    component n2 = Num2Bits(3);
    n2.in <== b;

    lt.in[0] <== a;
    lt.in[1] <== b;
    out <== lt.out;
}

component main = VulnerableLessThan();
