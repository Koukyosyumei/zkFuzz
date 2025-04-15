#!/bin/bash

cd tools/circomspect
cargo build --release
cp ./target/release/circomspect /usr/local/bin/circomspect