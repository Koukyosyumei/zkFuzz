pragma circom 2.1.6;

include "../CommitmentCircuit/circuit.circom";
include "../MerkleTreeInclusionCircuit/circuit.circom";
include "./Vote.circom";

template ZkSnarkProof(num){
    // Private
    signal input r;
    signal input c_id;
    signal input v;
    signal input t_id;

    // Public
    signal input pk_t;
    signal input g;
    signal input e_v[2];
    signal input cr_id;
    signal input pathIndices[num];
    signal input siblings[num];
    signal input root;

    // Output
    signal output out;

    component vote = Vote();
    var vote_check;
    vote.pk_t <== pk_t;
    vote.g <== g;
    vote.e_v[0] <== e_v[0];
    vote.e_v[1] <== e_v[1];
    vote.r <== r;
    vote.v <== v;
    vote_check = vote.out;

    component com = PoseidonCommitment();
    var com_check;
    com.c_id <== c_id;
    com.cr_id <== cr_id;
    com.t_id <== t_id;
    com_check = com.out;

    component mtip = MerkleTreeInclusionProof(num);
    var mtip_check;
    mtip.leaf <== c_id;
    mtip.root <== root;
    for (var i = 0; i < num; i++) {
        mtip.pathIndices[i] <== pathIndices[i];
        mtip.siblings[i] <== siblings[i];
    }
    mtip_check = mtip.out;

    component nullVote = Vote();
    var null_vote_check;
    nullVote.pk_t <== pk_t;
    nullVote.g <== g;
    nullVote.e_v[0] <== e_v[0];
    nullVote.e_v[1] <== e_v[1];
    nullVote.r <== r;
    nullVote.v <== 0;
    null_vote_check = nullVote.out;

    var vote_com;
    var part_one;
    vote_com = vote_check && com_check ? 1 : 0;
    part_one = vote_com && mtip_check ? 1 : 0;

    // log("vote_check: ", vote_check);
    // log("com_check: ", com_check);
    // log("vote_com: ", vote_com);
    // log("mtip_check: ", mtip_check);
    // log("part_one: ", part_one);
    // log("null_vote_check: ", null_vote_check);
    out <-- part_one || null_vote_check ? 1 : 0;
    out === 1;

    signal dummy;
    dummy <== t_id * r;

}
// component main {public [pk_t, g, e_v, cr_id, pathIndices, siblings, root]} = ZkSnarkProof(3);
