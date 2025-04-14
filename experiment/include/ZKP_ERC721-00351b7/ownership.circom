pragma circom 2.0.0;

include "../circomlib-cff5ab6/poseidon.circom";

template OwnershipFunction() {
  signal input tokenId;
  signal input salt;
  signal output hashOfTokenId;

  component poseidon1 = Poseidon(1);
  poseidon1.inputs[0] <== tokenId;

  component poseidon2 = Poseidon(1);
  poseidon2.inputs[0] <== poseidon1.out + salt;
  hashOfTokenId <== poseidon2.out;
}

// component main {public [tokenId]} = OwnershipFunction();