#!/bin/bash

# default values
TARGET_DIR="./benchmarks1/circomlib-cff5ab6"
TIME_LIMIT=60
MODE="zkfuzz"
RECURSION_DEPTH=1000

while getopts d:t:m:r: OPT; do
  case $OPT in
  "d")
    FLG_D="TRUE"
    TARGET_DIR="$OPTARG"
    ;;
  "t")
    FLG_T="TRUE"
    TIME_LIMIT="$OPTARG"
    ;;
  "m")
    FLG_M="TRUE"
    MODE="$OPTARG"
    ;;
  "r")
    FLG_R="TRUE"
    RECURSION_DEPTH="$OPTARG"
  esac
done

if [ "${MODE}" = "zkfuzz" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        timeout $TIME_LIMIT zkfuzz "$circom_file" --search_mode="ga" --save_output
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
    done
elif [ "${MODE}" = "zkfuzz-oc" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        timeout $TIME_LIMIT zkfuzz "$circom_file" --search_mode="ga" --constraint_assert_dissabled --save_output
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
    done
elif [ "${MODE}" = "conscs" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        dirpath=$(dirname "$circom_file")
        circom $circom_file --r1cs -o $dirpath > /dev/null
        no_ext="${circom_file%.circom}"
        timeout $TIME_LIMIT python3 ./tools/ConsCS/analyze_circuit.py "${no_ext}.r1cs" ./tools/ConsCS/logs/test_log.log ./tools/ConsCS/logs/test_log_with_contributions.log 111 4
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
    done
elif [ "${MODE}" = "zkap" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        { time timeout $TIME_LIMIT zkap "$circom_file" 2> "${circom_file}.txt"; } 2> temp_time.txt
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
        grep real temp_time.txt > "${circom_file}_time.txt"
    done
    rm temp_time.txt
elif [ "${MODE}" = "picus-z3" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        { time timeout $TIME_LIMIT ./tools/Picus/run-picus --solver z3 "$circom_file" > "${circom_file}.txt"; } 2> temp_time.txt
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
        grep real temp_time.txt > "${circom_file}_time.txt"
    done
    rm temp_time.txt
elif [ "${MODE}" = "picus-cvc5" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        { time timeout $TIME_LIMIT ./tools/Picus/run-picus --solver cvc5 "$circom_file" > "${circom_file}.txt"; } 2> temp_time.txt
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
        grep real temp_time.txt > "${circom_file}_time.txt"
    done
    rm temp_time.txt
elif [ "${MODE}" = "circomspect" ]; then
    for circom_file in "$TARGET_DIR"/*.circom; do
        echo "Processing: $circom_file"
        { time timeout $TIME_LIMIT circomspect "$circom_file" -d "$RECURSION_DEPTH" > "${circom_file}.txt"; } 2> temp_time.txt
        if [ $? -eq 124 ]; then
            echo "Timeout reached for file: $circom_file"
        fi
        grep real temp_time.txt > "${circom_file}_time.txt"
    done
    rm temp_time.txt
fi
