#include "llvm/Pass.h"
#include "utils_basicinfo.hpp"

// Definition

using namespace llvm;

using PFGNodeType = ValueTypeEnum;

std::string nodeTypeEnumToAbbr(PFGNodeType ne);

enum class PFGEdgeType {
    Assignment,
    Constraint,
};

std::string edgeTypeEnumToAbbr(PFGEdgeType ee);

class PFGNode;
class PFGEdge;
class PFGraph;

using NodeVec = std::vector<PFGNode *>;
using EdgeVec = std::vector<PFGEdge *>;
using GraphVec = std::vector<PFGraph *>;
using NodeMap = std::unordered_map<std::string, PFGNode *>;
using EdgeMap = std::unordered_map<std::string, PFGEdge *>;
using GraphMap = std::unordered_map<std::string, PFGraph *>;

class PFGNode {
   private:
   public:
    PFGNodeType type;
    std::string name;
    std::string var_name;
    Instruction *inst;
    // this == edge->_from;
    std::vector<PFGEdge *> flowto;
    // this == edge->_to;
    std::vector<PFGEdge *> flowfrom;
    PFGNode(PFGNodeType type, std::string name, std::string var_name,
            Instruction *inst);
    void addEdge(PFGEdge *edge);
    bool operator==(const PFGNode &b);
    bool operator!=(const PFGNode &b);
    static std::string getHash(PFGNodeType type, std::string name);
    std::string getHash();
    std::string getName();
    std::string format();
    bool isSignal();
};

class PFGEdge {
   private:
   public:
    PFGEdgeType type;
    PFGNode *from;
    PFGNode *to;
    PFGEdge(PFGEdgeType type, PFGNode *from, PFGNode *to);
    bool operator==(const PFGEdge &b);
    bool operator!=(const PFGEdge &b);
    static std::string getHash(PFGEdgeType type, PFGNode *from, PFGNode *to);
    std::string getHash();
    std::string getName();
    std::string format();
};

class PFGraph {
   private:
    void buildSelfSignalNodes(NameSet signal_names, PFGNodeType node_ty);
    void buildCompSignalNodes();
    NodeVec buildCompSignalNodes(PFGraph *sub_graph, std::string comp_var_name,
                                 NameSet signal_names, PFGNodeType node_ty);
    void build();

    PFGNode *locateSourceNode(Value *v);
    PFGNode *locateDestinationNode(Value *v);
    NodeVec getNodes(LLVMValueAccess lv);
    NodeVec flatExpression(Value *v);
    NameSet flowsTo(PFGNode *n);

   public:
    bool is_instantiation;
    bool is_hacking;
    GraphMap global_graphs;
    Collector *collector;
    NodeMap nodes;
    NameIdx node_idxes;
    EdgeMap edges;

    UnionFind *uf;
    NameSetMap nodeFlowsTo;

    PFGraph(GraphMap global_graphs, Collector *collector);
    std::string getName();
    PFGNode *createNode(PFGNodeType type, std::string name,
                        std::string var_name, Instruction *inst);
    PFGEdge *createEdge(PFGEdgeType type, PFGNode *from, PFGNode *to);
    PFGNode *getNode(PFGNodeType type, std::string name);
    NodeVec getNodes(PFGNodeType type, std::string signal_name);
    size_t getNodeIndex(PFGNode *n);
    void compute();
    json::Object format();
    bool isDepended(std::string node_hash_a, std::string node_hash_b);
    bool isConstrained(std::string node_hash_a, std::string node_hash_b);

    ArrayShape getArrayShape(std::string var_name);

    // For detectors

    // UCO Confirmed
    bool constrainedByInput(PFGNode *n);
    bool constrainedAsConst(PFGNode *n);
    bool unconstrainedOutput(PFGNode *n);
    NameVec detectUnconstrainedOutput();

    // USCI Confirmed
    bool unconstrainedCompInput(PFGNode *n);
    NameVec detectUnconstrainedCompInput();

    // DCD Confirmed
    NameVec detectDataflowConstraintDis();

    // USCO Confirmed
    bool isCheckingSignal(PFGNode *n);
    bool unusedCompOutput(PFGNode *n);
    NameVec detectUnusedCompOutput();

    // UCS Confirmed
    bool unusedSignal(PFGNode *n);
    NameVec detectUnusedSignal();

    // DBZ ( I think taint analysis could work )
    bool isDenominatorWithSignal(Instruction *inst);
    bool isDivideByZeroUnsafe(PFGNode *n);
    NameVec detectDivideByZeroUnsafe();

    // NDD ( I think taint analysis could work )
    bool isBranchCondWithSignal(Instruction *inst);
    bool isBranchCondWithSignal(PFGNode *n);
    NameVec detectNondeterministicDataflow();
    // bool isMultipleExpression(Instruction *inst);

    // TM Todo
    NameVec detectTypeMismatch();

    // IR Confirmed
    bool isTrivialInstruction(Value *v);
    bool isRewritableAssignment(PFGEdge *e);
    NameVec detectAssignmentMisuse();

    // NC (Deprecated)
    NameVec detectNonuniformConstraint();

    // QC (Deprecated)
    bool isQuadricExpression(PFGNode *n);
    bool isUnconstrainedSignalWithoutQC(PFGNode *n);
    NameVec detectUnconstrainedSignalWithoutQC();
};

GraphVec initDetectedGraphs(Module &M, bool compute, bool only_main);
