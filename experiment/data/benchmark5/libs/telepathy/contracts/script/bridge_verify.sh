source .env
source ../../.env

echo "Source chain" ${SOURCE_CHAIN_ID}
SOURCE_AMB_VAR="SourceAMB_ADDRESS_${SOURCE_CHAIN_ID}"
SOURCE_TOKEN_VAR="InfiniteMintSuccincts_ADDRESS_${SOURCE_CHAIN_ID}"
SOURCE_DEPOSIT_VAR="Deposit_ADDRESS_${SOURCE_CHAIN_ID}"
SOURCE_ETHERSCAN_VAR="ETHERSCAN_API_KEY_${SOURCE_CHAIN_ID}"
RANDOM_ADDRESS="0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496"

forge verify-contract \
    --chain-id ${SOURCE_CHAIN_ID} \
    --num-of-optimizations 200 \
    --watch \
    ${!SOURCE_AMB_VAR} \
    src/bridge/Tokens.sol:InfiniteMintSuccincts \
    --constructor-args \
    $(cast abi-encode "constructor(uint256,address)" "0" "${RANDOM_ADDRESS}") \
    ${!SOURCE_ETHERSCAN_VAR}

forge verify-contract \
    --chain-id ${SOURCE_CHAIN_ID} \
    --num-of-optimizations 200 \
    --watch \
    ${!SOURCE_DEPOSIT_VAR} \
    src/bridge/Bridge.sol:Deposit \
    --constructor-args \
    $(cast abi-encode "constructor(address,address)" "${!SOURCE_AMB_VAR}" "${!SOURCE_TOKEN_VAR}") \
    ${!SOURCE_ETHERSCAN_VAR}

for CHAIN_ID in ${CHAIN_IDS[@]}
do
    echo "Destination chain" ${CHAIN_ID}
    WITHDRAW_VAR="Withdraw_ADDRESS_${CHAIN_ID}"
    AMB_VAR="TargetAMB_ADDRESS_${CHAIN_ID}"
    ETHERSCAN_VAR="ETHERSCAN_API_KEY_${CHAIN_ID}"
    # https://unix.stackexchange.com/questions/32218/dereference-concatenated-variable-name

    forge verify-contract \
        --chain-id ${CHAIN_ID} \
        --num-of-optimizations 200 \
        --watch \
        ${!WITHDRAW_VAR} \
        src/bridge/Bridge.sol:Withdraw \
        --constructor-args \
        $(cast abi-encode "constructor(address,address)" "${!AMB_VAR}" "${!SOURCE_DEPOSIT_VAR}") \
        ${!ETHERSCAN_VAR}
done