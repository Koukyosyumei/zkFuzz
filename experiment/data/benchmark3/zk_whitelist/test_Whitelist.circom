pragma circom 2.1.6;

template IsZero() {
    signal input in;
    signal output out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    out <== -in*inv +1;
    in*out === 0;
}


template IsEqual() {
    signal input in[2];
    signal output out;

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    isz.out ==> out;
}

template Whitelist () {
    
    signal input addressInDecimal;
    signal input sameAddressButPublic;
    signal output c;

    component ise = IsEqual();
    
    ise.in[0] <== addressInDecimal;
    ise.in[1] <== sameAddressButPublic;
    assert(ise.out==1);
    c <== ise.out;
    
}

component main { public [ sameAddressButPublic ] } = Whitelist();