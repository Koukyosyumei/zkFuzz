pragma circom 2.0.0;
template IfElse(C) {
    signal input a;
    signal input b;
    signal output out;
    if (C > 3) {
        out <-- a / b;
        b*out === a;
    } else {
        out <-- 2 * a / b;
        b*out === 2*a;
    }
}
component main = IfElse(2);
