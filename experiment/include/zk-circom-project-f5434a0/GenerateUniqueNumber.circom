pragma circom  2.0.0;

include "../circomlib-cff5ab6/bitify.circom";
include "../circomlib-cff5ab6/comparators.circom";
include "../circomlib-cff5ab6/mux1.circom";

template GenerateUniqueNumber(n){
    signal input in[n];
    signal input dob;
    signal output out;

    signal intermidate[n];
    signal sum[n+1];

    sum[0] <== 0;
    for (var i = 0; i < n; i++){
        intermidate[i] <== in[i] * (i+1);
        sum[i+1] <== sum[i] + intermidate[i];
    }
    
    component customHash = calculateHash12();
    customHash.in1 <== sum[n] + dob;

    log(customHash.out);
}

template calculateHash12(){
    signal input in1;
    signal output out;
    
    component n2b = Num2Bits(254);
    component lessThan = LessThan(40);
    component mux = Mux1();

    n2b.in <== in1;

    var lc = 0;
    var e = 1;
    for (var i = 0; i < 40; i++){
        lc += n2b.out[i] * e;
        e = e * 2;
    }

    signal lcSignal <== lc;

    lessThan.in[0] <== lcSignal;
    lessThan.in[1] <== 900000000000;

    mux.c[0] <==  lcSignal;
    mux.c[1] <== lcSignal - 900000000000;
    mux.s <== lessThan.out;

    out <== 1000000000000 + mux.out;
}

// component main = GenerateUniqueNumber(4);