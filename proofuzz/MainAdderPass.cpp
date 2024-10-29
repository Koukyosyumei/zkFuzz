#include <regex>
#include <unordered_map>

#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/Pass.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"

#include "ExtendedProtocolFlowGraph.hpp"
#include "helpers.hpp"

using namespace llvm;

static cl::opt<bool> OverwriteFreeVariable("enable-overwrite-free-variables", cl::desc("Enable arbitrary assignments to free variables"));
static cl::opt<bool> PrintoutOutputs("printout-outputs", cl::desc("Print out all outputs of the main circuits"));
static cl::opt<bool> PrintoutConstraints("printout-constraints", cl::desc("Print out the logical AND of all constraints of the main circuits"));

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
        FunctionCallee printfFunc, scanfFunc, exitFunc;
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
            exitFunc = declareExitFunction(M);

            // Find the inputs and outputs of the circuit using pattern matching
            for (Function &F : M)
            {
                if (F.getName().startswith("fn_template_init"))
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

            // Clone the target function
            cloneFunctions(M, "fn_template_init_" + circuitName, "cloned_");

            // Remove the store instruction to the free intermediate/output variables.
            if (OverwriteFreeVariable)
            {
                overwriteStoreToFreeVariables(M, "cloned_fn_template_init_" + circuitName, circuitName);
            }

            // Declare the `main` function that initializes an instance of the target circuit.
            createMainFunction(M);

            return true;
        }

        void overwriteStoreToFreeVariables(Module &M, const std::string &funcName, const std::string &circuitName)
        {
            LLVMContext &Context = M.getContext();
            Function *F = M.getFunction(funcName);

            EPFGraph *g = nameToGraph[circuitName];
            std::vector<Instruction *> toInsert, toRemove;

            for (auto p : g->nodes)
            {
                PFGNode *n = p.second;
                if (g->isFree(n))
                {
                    findAllocas(F, "initial." + n->getName().substr(1) + ".*", toInsert);
                    findStores(F, "initial." + n->getName().substr(1) + ".*", toRemove);
                }
            }

            // Overwrite free variables
            for (auto &Arg : F->args())
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
            Constant *formatStrInt = ConstantDataArray::getString(Context, "%d\n", true);
            Constant *formatStrLongInt = ConstantDataArray::getString(Context, "%ld\n", true);
            Constant *formatStrLongLongInt = ConstantDataArray::getString(Context, "%lld", true);

            // Define the `main` function type and create the function
            FunctionType *mainType = FunctionType::get(Builder.getInt32Ty(), false);
            Function *mainFunc = Function::Create(mainType, Function::ExternalLinkage, "main", M);

            // Create the basic block for the `main` function
            BasicBlock *entry = BasicBlock::Create(Context, "entry", mainFunc);
            Builder.SetInsertPoint(entry);

            GlobalVariable *formatStrIntVar = new GlobalVariable(
                M, formatStrInt->getType(), true, GlobalValue::PrivateLinkage, formatStrInt, ".str.map.d");
            GlobalVariable *formatStrLongIntVar = new GlobalVariable(
                M, formatStrLongInt->getType(), true, GlobalValue::PrivateLinkage, formatStrLongInt, ".str.map.ld");
            GlobalVariable *formatStrLongLongIntVar = new GlobalVariable(
                M, formatStrLongLongInt->getType(), true, GlobalValue::PrivateLinkage, formatStrLongLongInt, ".str.map.lld");

            Function *buildFuncPtr = M.getFunction("fn_template_build_" + circuitName);
            Value *instance = Builder.CreateCall(buildFuncPtr, {}, "instance");

            // Read inputs from standard inputs
            for (const std::pair<std::string, int> kv : gepInputIndexMap)
            {
                Value *inputPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                read128bit(Context, Builder, inputPtr, scanfFunc, formatStrLongLongIntVar);
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
                    read128bit(Context, Builder, fvPtr, scanfFunc, formatStrLongLongIntVar);
                }
            }

            Function *clonedFunc = nullptr;
            Value *outputPtrCloned = nullptr;
            AllocaInst *IsClonedSatisfyConstraintsAlloca = nullptr;

            if (OverwriteFreeVariable)
            {
                clonedFunc = M.getFunction("cloned_fn_template_init_" + circuitName);
                IsClonedSatisfyConstraintsAlloca = Builder.CreateAlloca(Type::getInt1Ty(Context),
                                                                        nullptr, "is_cloned_satisfy_constraints");
                if (clonedFunc)
                {
                    // Call the cloned circuit
                    Builder.CreateCall(clonedFunc, {instance});

                    // Load and print outputs
                    for (const std::pair<std::string, int> kv : gepOutputIndexMap)
                    {
                        Value *gepPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                        outputPtrCloned = Builder.CreateLoad(Builder.getInt128Ty(), gepPtr, ("cloned_result." + kv.first).c_str());
                        if (PrintoutOutputs)
                        {
                            print128bit(Context, Builder, outputPtrCloned, printfFunc, formatStrLongIntVar);
                        }
                    }

                    // Check if the constraints are satisfied
                    Value *InitialValue = ConstantInt::getTrue(Context);
                    Builder.CreateStore(InitialValue, IsClonedSatisfyConstraintsAlloca);
                    Value *NewResult;
                    for (auto &GV : M.globals())
                    {
                        if (GV.getName().startswith("constraint"))
                        {
                            Value *LoadedValue = Builder.CreateLoad(GV.getValueType(), &GV);
                            Value *CurrentResult = Builder.CreateLoad(Type::getInt1Ty(Context), IsClonedSatisfyConstraintsAlloca);
                            NewResult = Builder.CreateAnd(CurrentResult, LoadedValue);
                            Builder.CreateStore(NewResult, IsClonedSatisfyConstraintsAlloca);
                        }
                    }
                    Value *formatStrIntPtr = Builder.CreatePointerCast(formatStrIntVar, Type::getInt8PtrTy(Context));
                    if (PrintoutConstraints)
                    {
                        Builder.CreateCall(printfFunc, {formatStrIntPtr, NewResult});
                    }
                }
            }

            Function *initFunc = M.getFunction("fn_template_init_" + circuitName);
            Value *outputPtrOriginal = nullptr;
            AllocaInst *IsOriginalSatisfyConstraintsAlloca = Builder.CreateAlloca(Type::getInt1Ty(Context),
                                                                                  nullptr, "is_original_satisfy_constraints");
            if (initFunc)
            {
                // Call the original circuit
                Builder.CreateCall(initFunc, {instance});

                // Load and print outputs
                for (const std::pair<std::string, int> kv : gepOutputIndexMap)
                {
                    Value *gepPtr = getGEP(Context, Builder, instance, kv.second, kv.first.c_str());
                    outputPtrOriginal = Builder.CreateLoad(Builder.getInt128Ty(), gepPtr, ("original_result." + kv.first).c_str());
                    if (PrintoutOutputs)
                    {
                        print128bit(Context, Builder, outputPtrOriginal, printfFunc, formatStrLongIntVar);
                    }
                }

                // Check if the constraints are satisfied
                Value *InitialValue = ConstantInt::getTrue(Context);
                Builder.CreateStore(InitialValue, IsOriginalSatisfyConstraintsAlloca);
                Value *NewResult;
                for (auto &GV : M.globals())
                {
                    if (GV.getName().startswith("constraint"))
                    {
                        Value *LoadedValue = Builder.CreateLoad(GV.getValueType(), &GV);
                        Value *CurrentResult = Builder.CreateLoad(Type::getInt1Ty(Context), IsOriginalSatisfyConstraintsAlloca);
                        NewResult = Builder.CreateAnd(CurrentResult, LoadedValue);
                        Builder.CreateStore(NewResult, IsOriginalSatisfyConstraintsAlloca);
                    }
                }
                Value *formatStrIntPtr = Builder.CreatePointerCast(formatStrIntVar, Type::getInt8PtrTy(Context));
                if (PrintoutConstraints)
                {
                    Builder.CreateCall(printfFunc, {formatStrIntPtr, NewResult});
                }
            }

            if (OverwriteFreeVariable)
            {
                Value *outputNotEqual = Builder.CreateICmpNE(outputPtrCloned, outputPtrOriginal, "outputNotEqual");
                Value *originalConstraintValue = Builder.CreateLoad(Type::getInt1Ty(Context), IsOriginalSatisfyConstraintsAlloca, "originalConstraintValue");
                Value *clonedConstraintValue = Builder.CreateLoad(Type::getInt1Ty(Context), IsClonedSatisfyConstraintsAlloca, "clonedConstraintValue");

                Value *UndercConstrainedCondition = Builder.CreateAnd(outputNotEqual, originalConstraintValue, "tmp_under_constrained_condition");
                UndercConstrainedCondition = Builder.CreateAnd(UndercConstrainedCondition, clonedConstraintValue, "final_under_constrained_condition");

                BasicBlock *CurrentBB = Builder.GetInsertBlock();
                Function *CurrentFunc = CurrentBB->getParent();
                BasicBlock *ErrorBB = BasicBlock::Create(Context, "under_constrained_error", CurrentFunc);
                BasicBlock *ContinueBB = BasicBlock::Create(Context, "no_under_constrained_continue", CurrentFunc);
                Builder.CreateCondBr(UndercConstrainedCondition, ErrorBB, ContinueBB);

                Builder.SetInsertPoint(ErrorBB);
                Value *ErrorMsg = Builder.CreateGlobalStringPtr("Error: Under-Constraint-Condition Met. Terminating program.\n");
                Builder.CreateCall(printfFunc, {ErrorMsg});

                Builder.CreateCall(exitFunc, {ConstantInt::get(Type::getInt32Ty(Context), 1)});
                Builder.CreateUnreachable();

                Builder.SetInsertPoint(ContinueBB);
            }

            Value *zeroVal = Builder.getInt32(0);
            Builder.CreateRet(zeroVal);
        }
    };
}

char MainAdderPass::ID = 0;
static RegisterPass<MainAdderPass> X("MainAdderPass", "Circom Transformation Pass", false, false);
