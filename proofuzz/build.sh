mkdir build
cd build
# cmake -DENABLE_LLVM_SHARED=1 ..
cmake ..
make

ls

clang++ -shared -fPIC -o libProoFuzzPass.so \
    $(llvm-config --cxxflags) \
    $(llvm-config --ldflags) \
    $(llvm-config --libs) \
    -Wl,--no-undefined \
    -I /usr/include/llvm \
    -L /usr/include/llvm \
    ../InitializeConstraintPass.cpp ../MainAdderPass.cpp ../ExtendedPrintGraphviz.cpp ../ExtendedProtocolFlowGraph.cpp ../../zkap/detectors/ProtocolFlowGraph.cpp ../../zkap/detectors/utils_arrayshapes.cpp ../../zkap/detectors/utils_basicinfo.cpp ../../zkap/detectors/utils.cpp
