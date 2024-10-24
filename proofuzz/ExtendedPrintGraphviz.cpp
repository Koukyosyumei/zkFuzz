#include "ExtendedProtocolFlowGraph.hpp"

void printEGraphviz(EPFGraph *graph)
{
    std::stringstream sb;
    sb << "digraph "
       << "\"" << graph->getName() << "\""
       << " {\n";
    sb << "graph [fontname=\"Helvetica\", fontsize=12, bgcolor=\"#f1f4f9\", style=\"filled\"];\n"; // Light subtle background
    sb << "node [shape=rectangle, style=\"filled,rounded\", fontname=\"Helvetica\", fontsize=10, penwidth=2];\n";
    sb << "edge [fontname=\"Helvetica\", fontsize=10, arrowsize=1.2, penwidth=1.5];\n"; // Stylish edges with bold weight

    for (auto p : graph->nodes)
    {
        PFGNode *n = p.second;
        bool isFree = graph->isFree(n);

        sb << "\"<<" << nodeTypeEnumToAbbr(n->type) << ">>\n"
           << n->getName();

        std::string fillcolor, fontcolor, shape, bordercolor;
        switch (n->type)
        {
        case PFGNodeType::Argument:
            fillcolor = "#6a8caf";   // Muted blue-gray
            fontcolor = "#ffffff";   // White for contrast
            bordercolor = "#4e6a86"; // Darker border for sophistication
            shape = "ellipse";
            break;
        case PFGNodeType::ComponentInput:
            fillcolor = "#e27d60";   // Warm coral orange
            fontcolor = "#ffffff";   // White for readability
            bordercolor = "#b8604b"; // Darker border for depth
            shape = "box";
            break;
        case PFGNodeType::ComponentOutput:
            fillcolor = "#7ea36d";   // Sage green
            fontcolor = "#ffffff";   // Light font for contrast
            bordercolor = "#5b7b4d"; // Darker green border
            shape = "box";
            break;
        case PFGNodeType::InputSignal:
            fillcolor = "#b786c5";   // Elegant lavender
            fontcolor = "#ffffff";   // White to pop against lavender
            bordercolor = "#8e6292"; // Rich purple border
            shape = "ellipse";
            break;
        case PFGNodeType::IntermediateSignal:
            fillcolor = "#a8d0e6";   // Light blue
            fontcolor = "#2c3e50";   // Dark blue-gray for clarity
            bordercolor = "#6c8ea4"; // Medium blue border for balance
            shape = "ellipse";
            break;
        case PFGNodeType::OutputSignal:
            fillcolor = "#b0e4d5";   // Cool mint
            fontcolor = "#000000";   // Black font for contrast
            bordercolor = "#7fb4a0"; // Darker mint border
            shape = "ellipse";
            break;
        case PFGNodeType::Expression:
            fillcolor = "#d1d1d1";   // Neutral light gray
            fontcolor = "#333333";   // Dark gray for legibility
            bordercolor = "#a0a0a0"; // Matching gray border
            shape = "diamond";
            break;
        case PFGNodeType::Constant:
            fillcolor = "#252525";   // Rich black
            fontcolor = "#f8f8f8";   // Almost white for sharp contrast
            bordercolor = "#555555"; // Slightly lighter black border
            shape = "hexagon";
            break;
        case PFGNodeType::Component:
            fillcolor = "#252525";   // Rich black
            fontcolor = "#f8f8f8";   // Almost white for sharp contrast
            bordercolor = "#555555"; // Slightly lighter black border
            shape = "hexagon";
            break;
        case PFGNodeType::Variable:
            fillcolor = "#252525";   // Rich black
            fontcolor = "#f8f8f8";   // Almost white for sharp contrast
            bordercolor = "#555555"; // Slightly lighter black border
            shape = "hexagon";
            break;
        }

        if (isFree)
        {
            fontcolor = "#e60000";
        }

        sb << "\" [fillcolor=\"" << fillcolor << "\", fontcolor=\"" << fontcolor << "\", color=\"" << bordercolor << "\", shape=\"" << shape << "\"];\n";
    }

    for (auto p : graph->edges)
    {
        auto e = p.second;
        auto left = e->from;
        auto right = e->to;
        // left
        sb << "\"<<" << nodeTypeEnumToAbbr(left->type) << ">>\n"
           << left->getName() << "\" -> ";
        // right
        sb << "\"<<" << nodeTypeEnumToAbbr(right->type) << ">>\n"
           << right->getName() << "\" ";
        //
        if (e->type == PFGEdgeType::Constraint)
        {
            sb << "[label=\"" << edgeTypeEnumToAbbr(e->type) << "\", dir=none, color=\"black:invis:black\", style=\"dashed\"];\n";
        }
        else if (e->type == PFGEdgeType::Assignment)
        {
            sb << "[label=\"" << edgeTypeEnumToAbbr(e->type) << "\", color=\"darkgreen\", arrowhead=\"vee\"];\n";
        }
    }
    sb << "}\n";
    std::cerr << sb.str();
}

namespace
{
    struct ExtendedPrintGraphviz : public ModulePass
    {
        static char ID;

        ExtendedPrintGraphviz() : ModulePass(ID) {}

        bool runOnModule(Module &M) override
        {
            EGraphVec graphs = initDetectedEGraphs(M, true, true);
            for (EPFGraph *g : graphs)
            {
                printEGraphviz(g);
            }
            return false;
        };
    };
} // namespace

char ExtendedPrintGraphviz::ID = 0;
static RegisterPass<ExtendedPrintGraphviz> X("ExtendedPrintGraphviz",
                                             "Print the graph in dot format.", false,
                                             false);
