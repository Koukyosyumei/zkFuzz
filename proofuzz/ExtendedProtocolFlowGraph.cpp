#include "ExtendedProtocolFlowGraph.hpp"

bool EPFGraph::isFree(PFGNode *n)
{
    if (n->type == PFGNodeType::IntermediateSignal || n->type == PFGNodeType::OutputSignal)
    {
        for (auto e : n->flowto)
        {
            if (e->type == PFGEdgeType::Constraint)
            {
                return false;
            }
        }
        for (auto e : n->flowfrom)
        {
            if (e->type == PFGEdgeType::Constraint)
            {
                return false;
            }
        }
        return true;
    }
    return false;
}

EGraphVec initDetectedEGraphs(Module &M, bool compute, bool only_main)
{
    auto graphs = EGraphVec();
    auto global_graphs = GraphMap();
    auto ordered_functions = sortFunctions(&M);
    auto ordered_collectors = sortCollectors(ordered_functions);
    auto main_comp = extractMainComp(&M);
    for (auto c : ordered_collectors)
    {
        auto graph = new EPFGraph(global_graphs, c);
        global_graphs.insert({graph->getName(), graph});
        if (compute)
        {
            graph->compute();
        }
        if (main_comp != "" && main_comp != c->getName() && only_main)
        {
            continue;
        }
        graphs.push_back(graph);
    }
    return graphs;
}