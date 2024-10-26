#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/Pass.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"
#include <regex>
#include <unordered_map>
#include <iostream>

#include "ExtendedProtocolFlowGraph.hpp"
#include "helpers.hpp"

using namespace llvm;

static cl::opt<bool> OverwriteFreeVariable("enable-overwrite-free-variables", cl::desc("Enable arbitrary assignments to free variables"));

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
        std::string circuitName;
        FunctionCallee printfFunc, scanfFunc;
        std::unordered_map<std::string, int> gepInputIndexMap, gepInterIndexMap, gepOutputIndexMap;
        std::unordered_map<std::string, EPFGraph *> nameToGraph;
        std::vector<std::string> freeVariableGEPNames;

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
            // Declare the `printf` and `scanf` function for output
            printfFunc = declarePrintfFunction(M);
            scanfFunc = declareScanfFunction(M);

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

            // Construct Extended Protocol Flow Graph
            EGraphVec graphs = initDetectedEGraphs(M, true, true);
            for (EPFGraph *g : graphs)
            {
                nameToGraph[g->getName()] = g;
            }

            if (OverwriteFreeVariable)
            {
                // Remove the store instruction to the free intermediate/output variables.
                overwriteStoreToFreeVariables(M);
            }

            // Declare the `main` function that initializes an instance of the target circuit.
            createMainFunction(M);

            return true;
        }

        void overwriteStoreToFreeVariables(Module &M)
        {
            LLVMContext &Context = M.getContext();

            for (Function &F : M)
            {
                if (F.getName().contains("fn_template_init"))
                {
                    std::string circuitName = F.getName().substr(17).str();
                    EPFGraph *g = nameToGraph[circuitName];
                    std::vector<Instruction *> toInsert, toRemove;

                    for (auto p : g->nodes)
                    {
                        PFGNode *n = p.second;
                        if (g->isFree(n))
                        {
                            findAllocas(&F, "initial." + n->getName().substr(1) + ".*", toInsert);
                            findStores(&F, "initial." + n->getName().substr(1) + ".*", toRemove);
                        }
                    }

                    // Overwrite free variables
                    for (auto &Arg : F.args())
                    {
                        if (Arg.getType()->isPointerTy())
                        {
                            Type *PtrTy = dyn_cast<Type>(Arg.getType());
                            if (StructType *StructTy = dyn_cast<StructType>(PtrTy->getPointerElementType()))
                            {
                                if (StructTy->getName().startswith("struct_template_"))
                                {
                                    for (auto *I : toInsert)
                                    {
                                        IRBuilder<> Builder(I->getNextNode());
                                        Value *valPtr = nullptr;
                                        std::string valName = I->getName().str();
                                        std::string gepName = "gep." + circuitName + "|" + valName.substr(8);
                                        freeVariableGEPNames.emplace_back(gepName);

                                        if (gepInterIndexMap.find(gepName) != gepInterIndexMap.end())
                                        {
                                            valPtr = getGEP(Context, Builder, &Arg, gepInterIndexMap[gepName], ("free." + gepName).c_str());
                                        }
                                        else if (gepOutputIndexMap.find(gepName) != gepOutputIndexMap.end())
                                        {
                                            valPtr = getGEP(Context, Builder, &Arg, gepOutputIndexMap[gepName], ("free." + gepName).c_str());
                                        }

                                        if (valPtr != nullptr)
                                        {
                                            Value *loadPtr = Builder.CreateLoad(Builder.getInt128Ty(), valPtr, ("free.read." + valName.substr(8)).c_str());
                                            Builder.CreateStore(loadPtr, dyn_cast<Value>(I));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Remove store instructions to free variables
                    for (auto *I : toRemove)
                    {
                        I->eraseFromParent();
                    }
                    break;
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

            // Define the constatn values
            Constant *formatStrScanf = ConstantDataArray::getString(Context, "%lld", true);
            Constant *formatStrPrintf = ConstantDataArray::getString(Context, "%ld\n", true);

            // Define the `main` function type and create the function
            FunctionType *mainType = FunctionType::get(Builder.getInt32Ty(), false);
            Function *mainFunc = Function::Create(mainType, Function::ExternalLinkage, "main", M);

            // Create the basic block for the `main` function
            BasicBlock *entry = BasicBlock::Create(Context, "entry", mainFunc);
            Builder.SetInsertPoint(entry);

            GlobalVariable *formatStrVar = new GlobalVariable(
                M, formatStrScanf->getType(), true, GlobalValue::PrivateLinkage, formatStrScanf, ".str.scanf");
            GlobalVariable *formatStrPrintfVar = new GlobalVariable(
                M, formatStrPrintf->getType(), true, GlobalValue::PrivateLinkage, formatStrPrintf, ".str.printf");

            // Execute the circuit and print outputs
            for (Function &F : M)
            {
                if (F.getName().contains("fn_template_build"))
                {
                    Value *instance = Builder.CreateCall(&F, {}, "instance");
                    unsigned index = 0;

                    // Read inputs from standard inputs
                    for (const std::pair<std::string, int> kv : gepInputIndexMap)
                    {
                        Value *inputPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                        Value *formatStrPtr = Builder.CreateBitCast(formatStrVar, Type::getInt8PtrTy(Context));
                        Builder.CreateCall(scanfFunc, {formatStrPtr, inputPtr});
                    }

                    // Read free variables from standard inputs
                    if (OverwriteFreeVariable)
                    {
                        for (const std::string fv_gep_name : freeVariableGEPNames)
                        {
                            Value *fvPtr = nullptr;
                            if (gepInterIndexMap.find(fv_gep_name) != gepInterIndexMap.end())
                            {
                                fvPtr = getGEP(Context, Builder, instance, gepInterIndexMap[fv_gep_name], fv_gep_name.c_str());
                            }
                            else if (gepOutputIndexMap.find(fv_gep_name) != gepOutputIndexMap.end())
                            {
                                fvPtr = getGEP(Context, Builder, instance, gepOutputIndexMap[fv_gep_name], fv_gep_name.c_str());
                            }
                            Value *formatStrPtr = Builder.CreateBitCast(formatStrVar, Type::getInt8PtrTy(Context));
                            Builder.CreateCall(scanfFunc, {formatStrPtr, fvPtr});
                        }
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

                        Value *formatStrPrintfPtr = Builder.CreatePointerCast(formatStrPrintfVar, Type::getInt8PtrTy(Context));
                        Builder.CreateCall(printfFunc, {formatStrPrintfPtr, highPart});
                        Builder.CreateCall(printfFunc, {formatStrPrintfPtr, lowPart});
                    }

                    break;
                }
            }

            Value *zeroVal = Builder.getInt32(0);
            Builder.CreateRet(zeroVal);
        }
    };
}

char MainAdderPass::ID = 0;
static RegisterPass<MainAdderPass> X("MainAdderPass", "Circom Transformation Pass", false, false);
