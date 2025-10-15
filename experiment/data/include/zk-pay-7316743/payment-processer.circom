pragma circom 2.1.6;

include "payment-amount-verifyer.circom";
include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/smt/smtprocessor.circom";

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

