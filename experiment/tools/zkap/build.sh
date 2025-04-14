#!/bin/bash

mkdir build
cd build
# cmake -DENABLE_LLVM_SHARED=1 ..
cmake ..
make

ls

clang++ -shared -fPIC -o libZKAPPass.so \
    $(llvm-config --cxxflags) \
    $(llvm-config --ldflags) \
    $(llvm-config --libs) \
    -Wl,--no-undefined \
    -I /usr/include/llvm \
    -L /usr/include/llvm \
    ../All.cpp ../PrintGraphviz.cpp ../PrintGraphInfo.cpp ../ProtocolFlowGraph.cpp ../utils_arrayshapes.cpp ../utils_basicinfo.cpp ../utils.cpp
sudo cp ./libZKAPPass.so /usr/local/lib/libZKAPPass.so
