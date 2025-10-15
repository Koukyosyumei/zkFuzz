pragma circom 2.1.6;
include "./elGamalcircuit.circom";

template Vote(){
    signal input pk_t;
    signal input g;
    signal input e_v[2];
    signal input r;
    signal input v;
    signal output out;

    component encrypt = Encrypt();

    encrypt.r <== r;
    encrypt.g <== g;
    encrypt.y <== pk_t;
    encrypt.m <== v;


    // e_v[0] === encrypt.c1;
    var e_v0;
    var e_v1;
    var vote_assert;
    e_v0 = e_v[0] == encrypt.c1 ? 1 : 0;
    e_v1 = e_v[1] == encrypt.c2 ? 1 : 0;
    vote_assert = e_v0 && e_v1 ? 1 : 0;
    // ev1 <== e_v[0] - encrypt.c1;
    //e_v[1] === encrypt.c2;
    // ev2 <== e_v[1] - encrypt.c2;

    out <-- vote_assert;

    signal dummy;
    dummy <== r * g;

}
//component main {public [pk_t, g, e_v]} = Vote();
