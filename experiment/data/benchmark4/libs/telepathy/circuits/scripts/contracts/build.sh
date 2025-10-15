cp ../build/step_cpp/verifier.sol StepVerifier.sol
sed -i 's/0\.6\.11/0\.8\.14/g' StepVerifier.sol
sed -i 's/Verifier/StepVerifier/g' StepVerifier.sol
sed -i 's/Pairing/PairingStep/g' StepVerifier.sol
sed -i 's/VerifyingKey/VerifyingKeyStep/g' StepVerifier.sol
sed -i 's/Proof/ProofStep/g' StepVerifier.sol
sed -i 's/verifyingKey(/verifyingKeyStep(/g' StepVerifier.sol
sed -i 's/verify(/verifyStep(/g' StepVerifier.sol
sed -i 's/verifyProof(/verifyProofStep(/g' StepVerifier.sol

cp ../build/rotate_cpp/verifier.sol RotateVerifier.sol
sed -i 's/0\.6\.11/0\.8\.14/g' RotateVerifier.sol
sed -i 's/Verifier/RotateVerifier/g' RotateVerifier.sol
sed -i 's/Pairing/PairingRotate/g' RotateVerifier.sol
sed -i 's/VerifyingKey/VerifyingKeyRotate/g' RotateVerifier.sol
sed -i 's/Proof/ProofRotate/g' RotateVerifier.sol
sed -i 's/verifyingKey(/verifyingKeyRotate(/g' RotateVerifier.sol
sed -i 's/verify(/verifyRotate(/g' RotateVerifier.sol
sed -i 's/verifyProof(/verifyProofRotate(/g' RotateVerifier.sol

cp StepVerifier.sol ../../contracts/src/lightclient/StepVerifier.sol
cp RotateVerifier.sol ../../contracts/src/lightclient/RotateVerifier.sol