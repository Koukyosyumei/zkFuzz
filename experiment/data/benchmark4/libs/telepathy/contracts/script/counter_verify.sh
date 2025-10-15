source .env
source ../../.env

echo "Source chain" ${SOURCE_CHAIN_ID}
SOURCE_AMB_VAR="SourceAMB_ADDRESS_${SOURCE_CHAIN_ID}"
SOURCE_COUNTER_VAR="Counter_ADDRESS_${SOURCE_CHAIN_ID}"
SOURCE_ETHERSCAN_VAR="ETHERSCAN_API_KEY_${SOURCE_CHAIN_ID}"

forge verify-contract \
    --chain-id ${SOURCE_CHAIN_ID} \
    --num-of-optimizations 200 \
    --watch \
    ${!SOURCE_AMB_VAR} \
    src/amb/SourceAMB.sol:SourceAMB \
    ${!SOURCE_ETHERSCAN_VAR}

forge verify-contract \
    --chain-id ${SOURCE_CHAIN_ID} \
    --num-of-optimizations 200 \
    --watch \
    ${!SOURCE_COUNTER_VAR} \
    test/counter/Counter.sol:Counter \
    --constructor-args \
    $(cast abi-encode "constructor(address,address,address)" "${!SOURCE_AMB_VAR}" "0x0000000000000000000000000000000000000000" "0x0000000000000000000000000000000000000000") \
    ${!SOURCE_ETHERSCAN_VAR}

for CHAIN_ID in ${CHAIN_IDS[@]}
do
    echo "Destination chain" ${CHAIN_ID}
    LIGHT_CLIENT_VAR="LightClient_ADDRESS_${CHAIN_ID}"
    COUNTER_VAR="Counter_ADDRESS_${CHAIN_ID}"
    AMB_VAR="TargetAMB_ADDRESS_${CHAIN_ID}"
    ETHERSCAN_VAR="ETHERSCAN_API_KEY_${CHAIN_ID}"
    # https://unix.stackexchange.com/questions/32218/dereference-concatenated-variable-name

    forge verify-contract \
        --chain-id ${CHAIN_ID} \
        --num-of-optimizations 200 \
        --watch \
        ${!LIGHT_CLIENT_VAR} \
        test/amb/LightClientMock.sol:LightClientMock \
        ${!ETHERSCAN_VAR}

    forge verify-contract \
        --chain-id ${CHAIN_ID} \
        --num-of-optimizations 200 \
        --watch \
        ${!AMB_VAR} \
        src/amb/TargetAMB.sol:TargetAMB \
        --constructor-args \
        $(cast abi-encode "constructor(address,address)" "${!LIGHT_CLIENT_VAR}" "${!SOURCE_AMB_VAR}") \
        ${!ETHERSCAN_VAR}

    forge verify-contract \
        --chain-id ${CHAIN_ID} \
        --num-of-optimizations 200 \
        --watch \
        ${!COUNTER_VAR} \
        test/counter/Counter.sol:Counter \
        --constructor-args \
        $(cast abi-encode "constructor(address,address,address)" "0x0000000000000000000000000000000000000000" "${!SOURCE_COUNTER_VAR}" "${!AMB_VAR}" ) \
        ${!ETHERSCAN_VAR}
done