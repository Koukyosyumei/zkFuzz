pragma circom 2.1.6;

include "../../include/passport-zk-circuits-9143bc7/vote/vote.circom";

component main {public [root, nullifierHash, vote]} = Vote(1);