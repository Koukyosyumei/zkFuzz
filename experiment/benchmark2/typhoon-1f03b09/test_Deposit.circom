pragma circom 2.0.0; 

include "../../include/typhoon-1f03b09/deposit.circom";

component main{public [oldRoot, rootNew, commitment, key]} = Deposit(8);