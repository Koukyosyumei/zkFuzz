#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/Pass.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"
#include <regex>
#include <unordered_map>
#include <iostream>

using namespace llvm;

namespace
{
    /**
     * @brief LLVM ModulePass that adds a `main` function to initialize and execute a circuit function
     * within the given LLVM module. It identifies inputs/outputs, calls the circuit initialization, and
     * processes the results by printing them using the `printf` function.
     */
    struct MainAdderPass : public ModulePass
    {
        static char ID;
        FunctionCallee printfFunc;
        std::unordered_map<std::string, int> gepInputIndexMap, gepInterIndexMap, gepOutputIndexMap;

        MainAdderPass() : ModulePass(ID) {}

        /**
         * @brief Runs the transformation pass on the given module.
         *
         * This function inserts a `main` function into the module, which sets up the circuit by calling
         * its initialization function and processing the input/output values.
         *
         * @param M The LLVM module to be transformed.
         * @return true Always returns true to indicate the module was modified.
         */
        bool runOnModule(Module &M) override
        {
            // Declare the `printf` function for output
            printfFunc = declarePrintfFunction(M);

            // Declare the `main` function that initializes an instance of the target circuit.
            createMainFunction(M);

            return true;
        }

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

        /**
         * @brief Finds all `alloca` instructions in a function that match a given name pattern.
         *
         * This method scans through all basic blocks and instructions within the specified function,
         * searching for `alloca` instructions (used for stack allocation) that match a specific regex pattern.
         *
         * @param F The function to search for `alloca` instructions.
         * @param pattern The regex pattern to match against instruction names.
         * @return A vector of pointers to matching `alloca` instructions.
         */
        std::vector<Instruction *> findAllocas(Function *F, const std::string &pattern)
        {
            std::vector<Instruction *> allocas;
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
            return allocas;
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

        /**
         * @brief Creates the `main` function to initialize and run the circuit within the LLVM module.
         *
         * This function sets up the main execution flow by identifying the circuit initialization function,
         * allocating input and output buffers, calling the necessary circuit functions, and printing results.
         *
         * @param M The LLVM module where the `main` function will be created.
         */
        void createMainFunction(Module &M)
        {
            LLVMContext &Context = M.getContext();
            IRBuilder<> Builder(Context);

            // Define the `main` function type and create the function
            FunctionType *mainType = FunctionType::get(Builder.getInt32Ty(), false);
            Function *mainFunc = Function::Create(mainType, Function::ExternalLinkage, "main", M);

            // Create the basic block for the `main` function
            BasicBlock *entry = BasicBlock::Create(Context, "entry", mainFunc);
            Builder.SetInsertPoint(entry);

            std::string circuitName;

            // Find the inputs and outputs of the circuit using pattern matching
            for (Function &F : M)
            {
                if (F.getName().contains("fn_template_init"))
                {
                    circuitName = F.getName().substr(17).str();

                    getIndexMap(&F, "gep.*.input", gepInputIndexMap);
                    getIndexMap(&F, "gep.*.inter", gepInterIndexMap);
                    getIndexMap(&F, "gep.*.output", gepOutputIndexMap);
                    break;
                }
            }

            Constant *formatStr = ConstantDataArray::getString(Context, "%ld\n", true);
            GlobalVariable *formatStrVar = new GlobalVariable(
                M, formatStr->getType(), true, GlobalValue::PrivateLinkage, formatStr, ".str");

            // Execute the circuit and print outputs
            for (Function &F : M)
            {
                if (F.getName().contains("fn_template_build"))
                {
                    Value *instance = Builder.CreateCall(&F, {}, "instance");
                    unsigned index = 0;

                    // Store inputs
                    for (const std::pair<std::string, int> kv : gepInputIndexMap)
                    {
                        Value *inputPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                        Builder.CreateStore(ConstantInt::get(Builder.getInt128Ty(), 123), inputPtr); // 123 is the example values
                    }

                    // Call circuit initialization
                    std::string initFuncName = "fn_template_init_" + circuitName;
                    Function *initFunc = M.getFunction(initFuncName);
                    if (initFunc)
                    {
                        Builder.CreateCall(initFunc, {instance});
                    }

                    // Load and print outputs
                    for (const std::pair<std::string, int> kv : gepOutputIndexMap)
                    {
                        Value *outputPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                        Value *outputVal = Builder.CreateLoad(Builder.getInt128Ty(), outputPtr, ("val." + kv.first).c_str());

                        Value *lowPart = Builder.CreateTrunc(outputVal, Type::getInt64Ty(Context));
                        Value *shifted = Builder.CreateLShr(outputVal, ConstantInt::get(Type::getInt128Ty(Context), 64));
                        Value *highPart = Builder.CreateTrunc(shifted, Type::getInt64Ty(Context));

                        Value *formatStrPtr = Builder.CreatePointerCast(formatStrVar, Type::getInt8PtrTy(Context));
                        Builder.CreateCall(printfFunc, {formatStrPtr, highPart});
                        Builder.CreateCall(printfFunc, {formatStrPtr, lowPart});
                    }

                    break;
                }
            }

            // Return based on the value of `constraint`
            // GlobalVariable *constraintVar = M.getGlobalVariable("constraint");
            // Value *constraintVal = Builder.CreateLoad(Builder.getInt1Ty(), constraintVar, "constraint_val");
            // Value *constraintI32 = Builder.CreateZExt(constraintVal, Builder.getInt32Ty(), "constraint_i32");
            // Builder.CreateRet(constraintI32);
            Value *zeroVal = Builder.getInt32(0);
            Builder.CreateRet(zeroVal);
        }
    };
}

char MainAdderPass::ID = 0;
static RegisterPass<MainAdderPass> X("MainAdderPass", "Circom Transformation Pass", false, false);
