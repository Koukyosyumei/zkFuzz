pragma circom 2.1.6;

include "../circomlib-cff5ab6/comparators.circom";

template Over21() {

    signal input age;
    signal output oldEnough;
    
    // 8 bits is plenty to store age
    component gt = GreaterThan(8);
    gt.in[0] <== age;
    gt.in[1] <== 21;
    
    oldEnough <== gt.out;
}