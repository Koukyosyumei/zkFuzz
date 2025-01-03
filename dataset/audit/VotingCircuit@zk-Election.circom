pragma circom 2.2.1;
// https://github.com/Dyslex7c/zk-Election/blob/bddc5c24315bf3dd7aeeb6611fa9936f1e23f733/circuits/voting_circuit.circom

// Correct import paths using circomlib
include "../circomlib/circuits/comparators.circom";
include "../circomlib/circuits/sha256/sha256.circom";
include "../circomlib/circuits/bitify.circom";
include "../circomlib/circuits/poseidon.circom";

// Main voting circuit
template VotingCircuit(NUM_CANDIDATES, VOTER_SECRET_BITS) {
    // Public inputs
    signal input electionId;  // Unique identifier for the election
    signal input candidateId; // Candidate being voted for
    
    // Private inputs
    signal input voterSecret[VOTER_SECRET_BITS];  // Voter's private key
    signal input nullifierSecret;  // Secret used to generate unique nullifier
    
    // Outputs
    signal output nullifier;  // Prevents double voting
    signal output commitment; // Allows verification without revealing voter identity
    
    // Candidate validation
    component candidateCheck = LessThan(32);
    candidateCheck.in[0] <== candidateId;
    candidateCheck.in[1] <== NUM_CANDIDATES;
    candidateCheck.out === 1;
    
    // Generate nullifier using Poseidon hash (efficient for zk circuits)
    component nullifierHasher = Poseidon(2);
    nullifierHasher.inputs[0] <== nullifierSecret;
    nullifierHasher.inputs[1] <== electionId;
    nullifier <== nullifierHasher.out;
    
    // Generate commitment using Poseidon hash with multiple steps
    component voterSecretHasher = Poseidon(2);
    voterSecretHasher.inputs[0] <== voterSecret[0];
    voterSecretHasher.inputs[1] <== voterSecret[1];
    
    component finalCommitmentHasher = Poseidon(2);
    finalCommitmentHasher.inputs[0] <== voterSecretHasher.out;
    finalCommitmentHasher.inputs[1] <== electionId;
    
    commitment <== finalCommitmentHasher.out;
    
    // Optional: Add additional constraints for vote validity
    // Example: Ensure vote is within valid range
    component rangeCheck = LessThan(32);
    rangeCheck.in[0] <== candidateId;
    rangeCheck.in[1] <== NUM_CANDIDATES;
    rangeCheck.out === 1;
}

// Tally circuit for vote counting
template VoteTallyCircuit(NUM_CANDIDATES) {
    // Public inputs
    signal input electionId;
    
    // Private inputs: array of votes and their proofs
    signal input votes[NUM_CANDIDATES];
    signal input commitments[NUM_CANDIDATES];
    signal input nullifiers[NUM_CANDIDATES];
    
    // Outputs
    signal output totalVotes[NUM_CANDIDATES];
    signal output electionResult;
    
    // Prevent duplicate nullifiers
    component nullifierUniqueness[NUM_CANDIDATES][NUM_CANDIDATES];
    for (var i = 0; i < NUM_CANDIDATES; i++) {
        for (var j = 0; j < NUM_CANDIDATES; j++) {
            if (i != j) {
                nullifierUniqueness[i][j] = IsEqual();
                nullifierUniqueness[i][j].in[0] <== nullifiers[i];
                nullifierUniqueness[i][j].in[1] <== nullifiers[j];
                nullifierUniqueness[i][j].out === 0;
            }
        }
        
        // Validate each vote
        totalVotes[i] <== votes[i];
    }
    
    // Determine election result (candidate with most votes)
    component resultDeterminer = MaxIndex(NUM_CANDIDATES);
    for (var i = 0; i < NUM_CANDIDATES; i++) {
        resultDeterminer.in[i] <== totalVotes[i];
    }
    electionResult <== resultDeterminer.out;
}

// Utility circuit for vote verification
template VoteVerificationCircuit() {
    // Inputs for vote verification
    signal input commitment;
    signal input nullifier;
    signal input electionId;
    signal input voterSecret;
    
    // Verification components
    component commitmentVerifier = Poseidon(2);
    commitmentVerifier.inputs[0] <== voterSecret;
    commitmentVerifier.inputs[1] <== electionId;
    
    // Ensure commitment matches
    commitmentVerifier.out === commitment;
    
    // Nullifier verification (prevents double voting)
    component nullifierVerifier = Poseidon(2);
    nullifierVerifier.inputs[0] <== voterSecret;
    nullifierVerifier.inputs[1] <== electionId;
    
    nullifierVerifier.out === nullifier;
}

// Helper circuit for MaxIndex (finding winner)
template MaxIndex(n) {
    signal input in[n];
    signal output out;
    
    component max[n-1];
    signal maxValue[n];
    
    maxValue[0] <== in[0];
    out <== 0;
    
    for (var i = 1; i < n; i++) {
        max[i-1] = GreaterThan(32);
        max[i-1].in[0] <== in[i];
        max[i-1].in[1] <== maxValue[i-1];
        
        maxValue[i] <== max[i-1].out ? in[i] : maxValue[i-1];
        out <== max[i-1].out ? i : out;
    }
}

// Main component instantiation
component main = VotingCircuit(5, 256);