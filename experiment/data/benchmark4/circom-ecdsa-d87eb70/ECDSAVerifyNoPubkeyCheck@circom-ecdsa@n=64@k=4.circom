pragma circom 2.0.0;
 
include "../libs/circom-ecdsa-d87eb70/ecdsa.circom";

component main = ECDSAVerifyNoPubkeyCheck(64, 4);
