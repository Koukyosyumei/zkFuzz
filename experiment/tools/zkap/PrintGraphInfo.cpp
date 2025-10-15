#include "ProtocolFlowGraph.hpp"

namespace {
struct PrintGraphInfo : public ModulePass {
    static char ID;

    PrintGraphInfo() : ModulePass(ID) {}

    bool runOnModule(Module &M) override {
        auto graphs = initDetectedGraphs(M, true, false);
        auto results = json::Object();
        for (auto g : graphs) {
            auto g_result = json::Object();
            g_result["info"] = g->collector->format();
            g_result["graph"] = g->format();
            auto report = json::Object();
            g_result["reports"] = obj2value(report);
            results[g->getName()] = obj2value(g_result);
        }
        json::OStream J(errs());
        J.value(obj2value(results));
        errs() << "\n";
        return false;
    };
};
}  // namespace

char PrintGraphInfo::ID = 0;
static RegisterPass<PrintGraphInfo> X(
    "PrintGraphInfo", "PrintGraphInfo whether the PFG works or not.", false,
    false);
