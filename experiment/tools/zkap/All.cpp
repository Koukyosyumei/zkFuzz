#include "ProtocolFlowGraph.hpp"

namespace {
struct All : public ModulePass {
    static char ID;

    All() : ModulePass(ID) {}

    bool runOnModule(Module &M) override {
        auto graphs = initDetectedGraphs(M, true, false);
        auto results = json::Object();
        auto main_comp = extractMainComp(&M);
        for (auto g : graphs) {
            auto g_result = json::Object();
            auto report = json::Object();
            report["uco"] = g->detectUnconstrainedOutput();
            report["usci"] = g->detectUnconstrainedCompInput();
            report["dcd"] = g->detectDataflowConstraintDis();
            report["usco"] = g->detectUnusedCompOutput();
            report["us"] = g->detectUnusedSignal();
            report["dbz"] = g->detectDivideByZeroUnsafe();
            report["ndd"] = g->detectNondeterministicDataflow();
            report["tm"] = g->detectTypeMismatch();
            report["am"] = g->detectAssignmentMisuse();
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

char All::ID = 0;
static RegisterPass<All> X(
    "All",
    "Detect whether every component is checked or not.",
    false, false);
