#!/bin/bash

git clone https://github.com/Koukyosyumei/circomspect
cd circomspect
cargo build --release
cp ./target/release/circomspect /usr/local/bin/circomspect