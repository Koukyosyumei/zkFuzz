pragma circom 2.1.6;
        
include "../circomlib/circuits/comparators.circom";

template PaymentAmountVerifyer () {

    signal input views;
    signal input secret;
    signal inv_denominator;
    signal secret_mul_denom;
    signal output out;
    
    var denominator = 1000;
    //Make sure the secret is more then 0 
    // NOTE: commented out assert for debug purpose
    //assert(secret > 0);

    // To prevent underflow
    //assert(views > 1000);

    inv_denominator <-- 1/denominator;
    
    component eq = IsEqual();
    eq.in[0] <== 1;
    eq.in[1] <== inv_denominator * denominator;
    
    secret_mul_denom <== secret * inv_denominator;
    out <== secret_mul_denom * views;

    //log("amount", out);
}

component main = PaymentAmountVerifyer();