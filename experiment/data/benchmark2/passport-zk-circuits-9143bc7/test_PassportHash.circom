pragma circom 2.1.6;

include "../../include/passport-zk-circuits-9143bc7/hasher/passportHash.circom";

component main = PassportHash(1, 512, 160);