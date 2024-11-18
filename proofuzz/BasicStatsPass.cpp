#include <iostream>
#include "ExtendedProtocolFlowGraph.hpp"

namespace
{
    struct BasicStatsPass : public ModulePass
    {
        static char ID;

        BasicStatsPass() : ModulePass(ID) {}

        bool runOnModule(Module &M) override
        {
            EGraphVec graphs = initDetectedEGraphs(M, true, true);
            for (EPFGraph *g : graphs)
            {
                uint numFreeNodes = 0;
                for (auto p : g->nodes)
                {
                    numFreeNodes += g->isFree(p.second);
                }
                std::cerr << g->getName() << "," << numFreeNodes << std::endl;
            }
            return false;
        };
    };
} // namespace

char BasicStatsPass::ID = 0;
static RegisterPass<BasicStatsPass> X("BasicStatsPass",
                                      "Print the basic statics of the circuit.", false,
                                      false);
