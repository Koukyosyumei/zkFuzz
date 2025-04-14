pragma circom 2.1.4;

include "../circomlib-cff5ab6/comparators.circom";

template inRange() {
    signal input x1;
    signal input y1;
    signal input x2;
    signal input y2;
    signal input r;

    signal output out;

    component comp = LessThan(32);
    signal x1Sq;
    signal y2Sq;
    signal distSum;
    x1Sq <== (x1 - x2) ** 2;
    y2Sq <== (y1 - y2) ** 2;
    distSum <== x1Sq + y2Sq;
    comp.in[0] <== distSum;
    comp.in[1] <== r**2;
    comp.out === 1;

    out <-- comp.out;
    out === 1;
}