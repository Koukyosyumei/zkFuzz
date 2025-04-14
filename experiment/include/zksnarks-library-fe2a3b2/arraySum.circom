pragma circom 2.1.4;

include "../circomlib-cff5ab6/comparators.circom";

template arraySum() {
    signal input list[5];
    signal input sum;

    signal output out;

    signal total;
    var temp = 0;
    for(var i=0; i<5; i++){
      temp += list[i];
    }
    total <== temp;

    component eq = IsEqual();
    eq.in[0] <== sum;
    eq.in[1] <== total;
    eq.out === 1;

    out <-- eq.out;
    out === 1;
}