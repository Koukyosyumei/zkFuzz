#include "helpers.hpp"

/**
 * @brief Declares an external `printf` function for formatted output.
 *
 * This method inserts a declaration of the C standard library's `printf` function into the module.
 *
 * @param M The LLVM module where the function will be declared.
 * @return A callable reference to the `printf` function.
 */
FunctionCallee declarePrintfFunction(Module &M)
{
    LLVMContext &Ctx = M.getContext();
    Type *PrintfArgType = Type::getInt8PtrTy(Ctx);
    FunctionType *PrintfType = FunctionType::get(Type::getInt32Ty(Ctx), PrintfArgType, true);
    return M.getOrInsertFunction("printf", PrintfType);
}

FunctionCallee declareScanfFunction(Module &M)
{
    LLVMContext &Ctx = M.getContext();
    Type *ScanfArgType = Type::getInt8PtrTy(Ctx);
    FunctionType *PrintfType = FunctionType::get(Type::getInt32Ty(Ctx), ScanfArgType, true);
    return M.getOrInsertFunction("scanf", PrintfType);
}

FunctionCallee declareExitFunction(Module &M)
{
    LLVMContext &Ctx = M.getContext();
    FunctionType *ExitType = FunctionType::get(Type::getVoidTy(Ctx), {Type::getInt32Ty(Ctx)}, false);
    return M.getOrInsertFunction("exit", ExitType);
}

/**
 * @brief Finds all `alloca` instructions in a function that match a given name pattern.
 *
 * This method scans through all basic blocks and instructions within the specified function,
 * searching for `alloca` instructions (used for stack allocation) that match a specific regex pattern.
 *
 * @param F The function to search for `alloca` instructions.
 * @param pattern The regex pattern to match against instruction names.
 * @param allocas A vector of pointers to matching `alloca` instructions.
 */
void findAllocas(Function *F, const std::string &pattern, std::vector<Instruction *> &allocas)
{
    std::regex regexPattern(pattern);

    // Iterate over all basic blocks and instructions within the function
    for (auto &BB : *F)
    {
        for (auto &I : BB)
        {
            if (AllocaInst *AI = dyn_cast<AllocaInst>(&I))
            {
                if (std::regex_search(AI->getName().str(), regexPattern))
                {
                    allocas.push_back(AI);
                }
            }
        }
    }
}

void findStores(Function *F, const std::string &pattern, std::vector<Instruction *> &stores)
{
    std::regex regexPattern(pattern);

    // Iterate over all basic blocks and instructions within the function
    for (auto &BB : *F)
    {
        for (auto &I : BB)
        {
            if (StoreInst *SI = dyn_cast<StoreInst>(&I))
            {
                Value *Ptr = SI->getPointerOperand();
                if (Ptr->hasName())
                {
                    if (std::regex_search(Ptr->getName().str(), regexPattern))
                    {
                        stores.push_back(SI);
                    }
                }
            }
        }
    }
}

/**
 * @brief Helper function to generate a GEP (GetElementPtr) instruction.
 *
 * This method creates a GEP instruction to calculate the address of an element in a data structure.
 *
 * @param Context The LLVM context.
 * @param Builder The IRBuilder used to insert the instruction.
 * @param instance The base pointer to the data structure.
 * @param index The index of the element to access.
 * @param name The name of the GEP instruction.
 * @return A pointer to the calculated element.
 */
Value *getGEP(LLVMContext &Context, IRBuilder<> &Builder, Value *instance, unsigned index, const char *name)
{
    return Builder.CreateGEP(instance->getType()->getPointerElementType(), instance,
                             {Builder.getInt32(0), Builder.getInt32(index)}, name);
}

void getIndexMap(Function *F, const std::string &pattern, std::unordered_map<std::string, int> &gepIndexMap)
{
    std::regex regexPattern(pattern);

    for (auto &BB : *F)
    {
        for (auto &I : BB)
        {
            if (auto *GEP = dyn_cast<GetElementPtrInst>(&I))
            {
                // get the last operand, which is the index of the field
                if (auto *CI = dyn_cast<ConstantInt>(GEP->getOperand(GEP->getNumOperands() - 1)))
                {
                    int fieldIndex = CI->getZExtValue();
                    const std::string gepName = GEP->getName().str();
                    if (std::regex_search(gepName, regexPattern))
                    {
                        gepIndexMap[gepName] = fieldIndex;
                    }
                }
            }
        }
    }
}

llvm::StoreInst *read128bit(LLVMContext &Context, IRBuilder<> &Builder, Value *inputPtr, FunctionCallee &scanfFunc, GlobalVariable *formatStrVar)
{
    AllocaInst *tempLow = Builder.CreateAlloca(Builder.getInt64Ty(), nullptr);
    AllocaInst *tempHigh = Builder.CreateAlloca(Builder.getInt64Ty(), nullptr);

    // Read lower 64 bits
    // Value *formatStrPtr = Builder.CreateBitCast(formatStr64, Type::getInt8PtrTy(Context));
    Value *formatStrPtr = Builder.CreateBitCast(formatStrVar, Type::getInt8PtrTy(Context));
    Builder.CreateCall(scanfFunc, {formatStrPtr, tempLow});
    // Read upper 64 bits
    Builder.CreateCall(scanfFunc, {formatStrPtr, tempHigh});
    // Load the 64-bit parts
    Value *lowVal = Builder.CreateLoad(Builder.getInt64Ty(), tempLow);
    Value *highVal = Builder.CreateLoad(Builder.getInt64Ty(), tempHigh);
    // Extend low to 128 bits
    Value *lowExtended = Builder.CreateZExt(lowVal, Builder.getInt128Ty());
    // Shift high and extend to 128 bits
    Value *highExtended = Builder.CreateZExt(highVal, Builder.getInt128Ty());
    Value *highShifted = Builder.CreateShl(highExtended, ConstantInt::get(Builder.getInt128Ty(), 64));
    // Combine low and high parts
    Value *fullValue = Builder.CreateOr(lowExtended, highShifted);

    // Builder.CreateCall(scanfFunc, {formatStrPtr, inputPtr});
    return Builder.CreateStore(fullValue, inputPtr);
}

void print128bit(LLVMContext &Context, IRBuilder<> &Builder, Value *outputPtr, FunctionCallee &printfFunc, GlobalVariable *formatStrVar)
{
    Value *lowPart = Builder.CreateTrunc(outputPtr, Type::getInt64Ty(Context));
    Value *shifted = Builder.CreateLShr(outputPtr, ConstantInt::get(Type::getInt128Ty(Context), 64));
    Value *highPart = Builder.CreateTrunc(shifted, Type::getInt64Ty(Context));

    Value *formatStrPrintfPtr = Builder.CreatePointerCast(formatStrVar, Type::getInt8PtrTy(Context));
    Builder.CreateCall(printfFunc, {formatStrPrintfPtr, highPart});
    Builder.CreateCall(printfFunc, {formatStrPrintfPtr, lowPart});
}

bool cloneFunctions(Module &M, const std::string &funcName, const std::string &prefix)
{
    bool Modified = false;
    Function *F = M.getFunction(funcName);
    if (F)
    {
        ValueToValueMapTy VMap;
        Function *ClonedF = CloneFunction(F, VMap, nullptr);
        ClonedF->setName(prefix + funcName);
        Modified = true;
    }
    return Modified;
}