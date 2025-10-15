include "../circomlib-cff5ab6/comparators.circom";
include "../circomlib-cff5ab6/poseidon.circom";
include "../circomlib-cff5ab6/smt/smtverifier.circom";

template Withdraw(nLevels) {
    // Public inputs
    signal input nullifier;
    signal input root;
	signal input address;

    // Private Inputs
	signal input secret;
	signal input siblings[nLevels];
	signal input key;

    // Poseidon hash for the Nullifier
	component nullifierCmp = Poseidon(2);
	nullifierCmp.inputs[0] <== key;
	nullifierCmp.inputs[1] <== secret;

	component nullifierCheck = IsEqual();
	nullifierCheck.in[0] <== nullifierCmp.out;
	nullifierCheck.in[1] <== nullifier;
	nullifierCheck.out === 1;

	component hash = Poseidon(2);
	hash.inputs[0] <== secret;
	hash.inputs[1] <== nullifierCmp.out;

    // Address cannot be zero
	component zeroAddressCheck = IsZero();
	zeroAddressCheck.in <== address;
	zeroAddressCheck.out === 0;

    //  SMT Verification
	component smtVerifier = SMTVerifier(nLevels);
	smtVerifier.enabled <== 1;
	smtVerifier.fnc <== 0;
	smtVerifier.root <== root;
	for (var i=0; i<nLevels; i++) {
		smtVerifier.siblings[i] <== siblings[i];
	}
	smtVerifier.oldKey <== 0;
	smtVerifier.oldValue <== 0;
	smtVerifier.isOld0 <== 0;
	smtVerifier.key <== key;
	smtVerifier.value <== hash.out;
}

// Instantiate the main component
// component main{public [nullifier, root, address]} = Withdraw(8);