pragma circom 2.1.6;

include 'payment-processer.circom';

template BulkPaymentProcesser(nLevels, nTxns){

  signal input userID;
  signal input viewsList[nTxns];
  signal input secretsList[nTxns];
  signal input amountsList[nTxns];
  signal input siblingsList[nTxns][nLevels];
  signal input oldHashList[nTxns];
  signal input userOldTxNonce;
  signal input oldRoot;
  signal input newRoot;
  signal input newNonce;


  var root = oldRoot;
  var nonce = userOldTxNonce;
  component paymentProcessers[nTxns];
  for(var i = 0; i< nTxns; i++){
    paymentProcessers[i] = PaymentProcesser(nLevels);
    paymentProcessers[i].userID <== userID;
    paymentProcessers[i].views <== viewsList[i];
    paymentProcessers[i].secret <== secretsList[i];
    paymentProcessers[i].paymentAmount <== amountsList[i];
    paymentProcessers[i].userOldTxNonce <== userOldTxNonces[i];
    paymentProcessers[i].siblings <== siblingsList[i];
    paymentProcessers[i].oldTxHash <== oldHashList[i];
    paymentProcessers[i].oldRoot <== root;
    // update the root variable with new root status
    root = paymentProcessers[i].newRoot;
    // update the nonce
    nonce += 1;

  }
  // chek accuracy of all txns
  newRoot === root;
  newNonce === nonce;
}

//@dev: NOTE: We configured it to take only 4 txns batch for now
// component main{public [userID, viewsList, oldRoot, userOldTxNonce, newRoot, newNonce]} = BulkPaymentProcesser(10, 4);


