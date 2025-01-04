pragma circom 2.1.6;

// https://github.com/ilvcs/zk-pay/blob/73167433fe33140ff45b1f76ad65be485ae7b805/circuits/payment-amount-verifyer.circom

include "../include/circomlib/circuits/comparators.circom";
include "../include/circomlib/circuits/poseidon.circom";
include "../include/circomlib/circuits/smt/smtprocessor.circom";

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

template PaymentProcesser(nLevels) {

    signal input userID;
    signal input views;
    signal input secret;
    signal input paymentAmount;
    signal input userOldTxNonce;
    signal input siblings[nLevels];
    signal input oldTxHash;
    signal input oldRoot;
    signal output newRoot;

    component paymentAmountChecker = PaymentAmountVerifyer();
    paymentAmountChecker.views <== views;
    paymentAmountChecker.secret <== secret;

    // check if the amount is correct
    paymentAmount === paymentAmountChecker.out;

    // Verify create SMT tree
    component smtprocessor = SMTProcessor(nLevels);
    component poseidon = Poseidon(2);
    poseidon.inputs[0] <== paymentAmount;
    poseidon.inputs[1] <== userOldTxNonce + 1;

    // log("newTxHash", poseidon.out);

    smtprocessor.fnc[0] <== 0;
    smtprocessor.fnc[1] <== 1;
    smtprocessor.oldRoot <== oldRoot;
    smtprocessor.siblings <== siblings;
    smtprocessor.oldKey <== userID;
    smtprocessor.oldValue <== oldTxHash;
    smtprocessor.isOld0 <== 0;
    smtprocessor.newKey <== userID;
    smtprocessor.newValue <== poseidon.out;

    // Output the newRoot
    newRoot <== smtprocessor.newRoot;

}

component main = PaymentProcesser(10);