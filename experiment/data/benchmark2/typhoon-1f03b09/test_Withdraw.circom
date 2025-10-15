pragma circom 2.0.0; 

include "../../include/typhoon-1f03b09/withdraw.circom";

component main{public [nullifier, root, address]} = Withdraw(8);