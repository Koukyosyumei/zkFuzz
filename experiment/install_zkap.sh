#!/bin/bash

cd ./tools/circom2llvm
cargo build --bin=circom2llvm --package=circom2llvm --release
sudo cp ./target/release/circom2llvm /usr/local/bin/circom2llvm
cd ../../

cd ./tools/zkap
sh ./build.sh
sudo cp ./zkap.sh /usr/local/bin/zkap
sudo chmod 777 /usr/local/bin/zkap
cd ../../