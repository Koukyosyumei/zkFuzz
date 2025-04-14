pragma circom 2.0.0;

template AND(n) {
    signal input a;
    signal output out;
    out <== a*n;
}

template OR(n) {
    signal input a;
    signal output out;
    out <== a + n - a*n;
}

template TEST(n) {
    signal input a;
    signal input b;
    signal output out;
    component and = AND(n);
    component or = OR(n);
    and.a = a;
    or.a = b;

    out <== and.out + or.out;
}
