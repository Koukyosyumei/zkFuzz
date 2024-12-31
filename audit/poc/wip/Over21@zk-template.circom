pragma circom 2.1.6;
// https://github.com/tea2x/zk-template/blob/0a813fd8701a6bc7d7debb754880769398699f62/circuits/over21/circuit.circom

include "../circomlib/circuits/comparators.circom";

template Over21() {

    signal input age;
    signal output oldEnough;
    
    // 8 bits is plenty to store age
    component gt = GreaterThan(8);
    gt.in[0] <== age;
    gt.in[1] <== 21;
    
    oldEnough <== gt.out;
}

component main = Over21();