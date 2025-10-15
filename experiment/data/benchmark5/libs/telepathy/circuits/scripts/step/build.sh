#!/bin/bash
CIRCUIT_NAME=step
PHASE1=`realpath /shared/powersOfTau28_hez_final_27.ptau`
CIRCUITS_DIR=`realpath ../../circuits`
BUILD_DIR=`realpath ../../build`
OUTPUT_DIR=`realpath "$BUILD_DIR"/"$CIRCUIT_NAME"_cpp`
NODE_PATH=`realpath ~/node/node`
NODE_CMD="$NODE_PATH --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc"
SNARKJS_PATH=`realpath ~/snarkjs/cli.js`
RAPIDSNARK_PATH=`realpath ~/rapidsnark/build/prover`

if [ ! -d "$BUILD_DIR" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "$BUILD_DIR"
fi

# echo "****COMPILING CIRCUIT****"
# start=`date +%s`
# circom "$CIRCUITS_DIR"/"$CIRCUIT_NAME".circom --O1 --r1cs --sym --c --output "$BUILD_DIR"
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****Running make to make witness generation binary****"
# start=`date +%s`
# make -C "$OUTPUT_DIR"
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****Executing witness generation****"
# start=`date +%s`
# "$OUTPUT_DIR"/"$CIRCUIT_NAME" input.json "$OUTPUT_DIR"/witness.wtns
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****Converting witness to json****"
# start=`date +%s`
# npx snarkjs wej "$OUTPUT_DIR"/witness.wtns "$OUTPUT_DIR"/witness.json
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****GENERATING ZKEY 0****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH zkey new "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$OUTPUT_DIR"/p1.zkey
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****CONTRIBUTE TO PHASE 2 CEREMONY****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH zkey contribute "$OUTPUT_DIR"/p1.zkey "$OUTPUT_DIR"/p2.zkey -n="First phase2 contribution" -e="some random text for entropy"
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****VERIFYING FINAL ZKEY****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH zkey verify "$BUILD_DIR"/"$CIRCUIT_NAME".r1cs "$PHASE1" "$OUTPUT_DIR"/p2.zkey
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****EXPORTING VKEY****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH zkey export verificationkey "$OUTPUT_DIR"/p2.zkey "$OUTPUT_DIR"/vkey.json
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****GENERATING PROOF FOR SAMPLE INPUT****"
# start=`date +%s`
# $RAPIDSNARK_PATH "$OUTPUT_DIR"/p2.zkey "$OUTPUT_DIR"/witness.wtns "$OUTPUT_DIR"/proof.json "$OUTPUT_DIR"/public.json
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH groth16 verify "$OUTPUT_DIR"/vkey.json "$OUTPUT_DIR"/public.json "$OUTPUT_DIR"/proof.json
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****EXPORTING SOLIDITY SMART CONTRACT****"
# start=`date +%s`
# $NODE_CMD $SNARKJS_PATH zkey export solidityverifier "$OUTPUT_DIR"/p2.zkey "$OUTPUT_DIR"/verifier.sol
# end=`date +%s`
# echo "DONE ($((end-start))s)"

echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s`
$NODE_CMD $SNARKJS_PATH groth16 verify "$OUTPUT_DIR"/vkey.json ../public.json ../proof.json
end=`date +%s`
echo "DONE ($((end-start))s)"
