#include "ProtocolFlowGraph.hpp"

void printGraphviz(PFGraph* graph) {
    std::stringstream sb;
    sb << "digraph "
       << "\"" << graph->getName() << "\""
       << " {\n";
    for (auto p : graph->nodes) {
        auto n = p.second;
        sb << "\"<<" << nodeTypeEnumToAbbr(n->type) << ">>\n" << n->name;
        std::string color;
        switch (n->type) {
            case PFGNodeType::Argument:
                color = "#889aa4";
                break;
            case PFGNodeType::ComponentInput:
                color = "#ca9a8a";
                break;
            case PFGNodeType::ComponentOutput:
                color = "#bccd81";
                break;
            case PFGNodeType::InputSignal:
                color = "#c7aaf6";
                break;
            case PFGNodeType::IntermediateSignal:
                color = "#f8edfc";
                break;
            case PFGNodeType::OutputSignal:
                color = "#d0fbe1";
                break;
            case PFGNodeType::Expression:
                color = "#cccccc";
                break;
            case PFGNodeType::Constant:
                color = "#000000";
                break;
            case PFGNodeType::Component:
                color = "#000000";
                break;
            case PFGNodeType::Variable:
                color = "#000000";
                break;
        }
        sb << "\" [color=\"" << color << "\"];\n";
    }

    for (auto p : graph->edges) {
        auto e = p.second;
        auto left = e->from;
        auto right = e->to;
        // left
        sb << "\"<<" << nodeTypeEnumToAbbr(left->type) << ">>\n"
           << left->name << "\" -> ";
        // right
        sb << "\"<<" << nodeTypeEnumToAbbr(right->type) << ">>\n"
           << right->name << "\" ";
        //
        if (e->type == PFGEdgeType::Constraint) {
            sb << "[";
            sb << "label=" << edgeTypeEnumToAbbr(e->type) << ", ";
            sb << "dir=none"
               << ", ";
            sb << "color=\"black:invis:black\"";
            sb << "]\n";
        } else if (e->type == PFGEdgeType::Assignment) {
            sb << "[";
            sb << "label=" << edgeTypeEnumToAbbr(e->type);
            sb << "]\n";
        }
    }
    sb << "}\n";
    std::cerr << sb.str();
}

namespace {
struct PrintGraphviz : public ModulePass {
    static char ID;

    PrintGraphviz() : ModulePass(ID) {}

    bool runOnModule(Module& M) override {
        auto graphs = initDetectedGraphs(M, true, true);
        for (auto g : graphs) {
            printGraphviz(g);
        }
        return false;
    };
};
}  // namespace

char PrintGraphviz::ID = 0;
static RegisterPass<PrintGraphviz> X("PrintGraphviz",
                                     "Print the graph in dot format.", false,
                                     false);
