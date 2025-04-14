#include <iostream>
#include <regex>
#include <string>
#include <unordered_map>
#include <vector>
#include <numeric>
#include <algorithm>

#include "llvm/IR/Instructions.h"
#include "llvm/IR/Value.h"

using namespace llvm;

const std::string array_shape_mark = "fn_intrinsic_utils_arraydim";
using ArrayShapeDefinition = CallInst;
using ValueVec = std::vector<Value *>;
using LLVMArrayShape = std::vector<Value*>;

using ArrayShape = std::vector<uint64_t>;
// Array size, array index
// Example: [64,64,64], [13,23,33]
using ArrayShapePair =
    std::pair<ArrayShape, ArrayShape>;

using ArrayShapeMap = std::unordered_map<std::string, ArrayShape>;

bool isArrayPtrTy(Type* ty);
bool isArrayShapeDefinedInst(Instruction* inst);
bool isArrayReturnFunction(Function* F);
std::pair<Value*, LLVMArrayShape> extractArrayShapeDefine(
    ArrayShapeDefinition* inst);
void printShape(LLVMArrayShape shape);
ArrayShape genArrayShape(ValueVec v);
ArrayShape LLVMtoConcreteArrayShape(LLVMArrayShape s);
uint64_t getArraySize(ArrayShape s);
uint64_t computeIndex(ArrayShapePair ap);
