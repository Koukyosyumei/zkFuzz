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
    Type *PrintfArgType = Type::getInt8PtrTy(Ctx);
    FunctionType *PrintfType = FunctionType::get(Type::getInt32Ty(Ctx), PrintfArgType, true);
    return M.getOrInsertFunction("scanf", PrintfType);
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