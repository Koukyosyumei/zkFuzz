#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/Pass.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"
#include <regex>
#include <unordered_map>
#include <iostream>

using namespace llvm;

FunctionCallee declarePrintfFunction(Module &M);
FunctionCallee declareScanfFunction(Module &M);
void findAllocas(Function *F, const std::string &pattern, std::vector<Instruction *> &allocas);
void findStores(Function *F, const std::string &pattern, std::vector<Instruction *> &stores);
Value *getGEP(LLVMContext &Context, IRBuilder<> &Builder, Value *instance, unsigned index, const char *name);
void getIndexMap(Function *F, const std::string &pattern, std::unordered_map<std::string, int> &gepIndexMap);