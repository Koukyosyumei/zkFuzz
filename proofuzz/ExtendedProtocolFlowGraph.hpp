#include "../zkap/detectors/ProtocolFlowGraph.hpp"

class EPFGraph: public PFGraph {
    using PFGraph::PFGraph;
    bool isFree(PFGNode *n);
};