include "../../circomlib/circuits/comparators.circom";
// https://github.com/kevinz917/zksnarks-library/blob/fe2a3b265d89e0a3a28e461547707f37eaf07f68/src/circuits/arraySum/arraySum.circom

template arraySum() {
    signal input list[5];
    signal private input sum;

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

component main = arraySum();