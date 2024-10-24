#include "../zkap/detectors/ProtocolFlowGraph.hpp"

class EPFGraph : public PFGraph
{
public:
    using PFGraph::PFGraph;
    bool isFree(PFGNode *n);
};

using EGraphVec = std::vector<EPFGraph *>;

EGraphVec initDetectedEGraphs(Module &M, bool compute, bool only_main);