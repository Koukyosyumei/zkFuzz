pragma circom 2.0.0;
// https://github.com/tinaaliakbarpour/Circom-examples/blob/d71c19c0b346c21038b5ae455642f9accd306843/secretsharing/secretsharing.circom

include "../circomlib/circuits/comparators.circom";

template ModP() {
    signal input num;
    signal input p;
    signal output out;

    /* 
    error: operator '<==' not allowed

    var _num_mod_p = num;
    while (_num_mod_p >= p) { // TODO add constrains
        _num_mod_p <== _num_mod_p - p; 
    }
    */

    // var _num_mod_p = num;
    // while (_num_mod_p >= p) { // TODO add constrains
    //     _num_mod_p = _num_mod_p - p; 
    // }

    // num = p * quotient + remainder
    signal remainder <-- num % p; // TODO "mod" needs more constraints
    signal quotient <-- num \ p ;

    num === p * quotient + remainder;
    out <== remainder;
}

component main = ModP();