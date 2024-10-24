cmake_minimum_required(VERSION 3.10)
project(ProoFuzzPass)

set(CMAKE_CXX_STANDARD 14)

find_package(LLVM REQUIRED CONFIG)
include_directories(${LLVM_INCLUDE_DIRS})
link_directories(${LLVM_LIBRARY_DIRS})
add_definitions(${LLVM_DEFINITIONS})

add_library(ProoFuzzPass MODULE
    InitializeConstraintPass.cpp
    MainAdderPass.cpp
    ExtendedPrintGraphviz.cpp
    ExtendedProtocolFlowGraph.cpp
    ../zkap/detectors/ProtocolFlowGraph.hpp
    ../zkap/detectors/utils_arrayshapes.cpp
    ../zkap/detectors/utils_basicinfo.cpp
    ../zkap/detectors/utils.cpp
)

target_link_libraries(ProoFuzzPass PRIVATE LLVMCore LLVMSupport LLVMIRReader LLVMInstCombine LLVMScalarOpts)

