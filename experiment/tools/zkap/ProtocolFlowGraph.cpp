#include "ProtocolFlowGraph.hpp"

#include <sstream>

std::string nodeTypeEnumToAbbr(PFGNodeType ne) { return typeEnumToAbbr(ne); }

std::string edgeTypeEnumToAbbr(PFGEdgeType ee) {
    switch (ee) {
        case PFGEdgeType::Assignment:
            return "assign";
        case PFGEdgeType::Constraint:
            return "constraint";
    }
}

PFGNode::PFGNode(PFGNodeType type, std::string name, std::string var_name,
                 Instruction *inst) {
    this->type = type;
    this->name = name;
    this->var_name = var_name;
    this->inst = inst;
    this->flowto = std::vector<PFGEdge *>();
    this->flowfrom = std::vector<PFGEdge *>();
}

void PFGNode::addEdge(PFGEdge *edge) {
    if (this == edge->from) {
        this->flowto.push_back(edge);
    } else {
        this->flowfrom.push_back(edge);
    }
}

bool PFGNode::operator==(const PFGNode &b) {
    return this->name == b.name && this->type == b.type;
}

bool PFGNode::operator!=(const PFGNode &b) {
    return !(this->name == b.name && this->type == b.type);
}

std::string PFGNode::getHash(PFGNodeType type, std::string name) {
    return nodeTypeEnumToAbbr(type) + ": " + name;
}

std::string PFGNode::getHash() {
    return PFGNode::getHash(this->type, this->name);
}

std::string PFGNode::getName() { return this->name; }

std::string PFGNode::format() {
    std::string s = this->getName();
    llvm::raw_string_ostream output(s);
    if (this->inst != nullptr) {
        s += "; inst:";
        this->inst->print(output);
    };
    return s;
}

bool PFGNode::isSignal() {
    return !(this->type == PFGNodeType::Constant ||
             this->type == PFGNodeType::Expression);
}

PFGEdge::PFGEdge(PFGEdgeType type, PFGNode *from, PFGNode *to) {
    this->type = type;
    this->from = from;
    this->to = to;
}

std::string PFGEdge::format() { return this->getName(); }

bool PFGEdge::operator==(const PFGEdge &b) {
    return *this->from == *b.from && *this->to == *b.to && this->type == b.type;
}

bool PFGEdge::operator!=(const PFGEdge &b) {
    return !(*this->from == *b.from && *this->to == *b.to &&
             this->type == b.type);
}

std::string PFGEdge::getHash(PFGEdgeType type, PFGNode *from, PFGNode *to) {
    std::string m;
    switch (type) {
        case PFGEdgeType::Assignment:
            m = " --> ";
            break;
        case PFGEdgeType::Constraint:
            m = " === ";
            break;
    }
    return from->getHash() + m + to->getHash();
}

std::string PFGEdge::getHash() {
    return PFGEdge::getHash(this->type, this->from, this->to);
}

std::string PFGEdge::getName() {
    std::string m;
    switch (type) {
        case PFGEdgeType::Assignment:
            m = " --> ";
            break;
        case PFGEdgeType::Constraint:
            m = " === ";
            break;
    }
    return this->from->getName() + m + this->to->getName();
}

PFGNode *PFGraph::locateSourceNode(Value *v) {
    if (isArgument(v)) {
        auto res = this->createNode(PFGNodeType::Argument,
                                    "arg" + v->getName().str(), "", nullptr);
        return res;
    }
    if (isConstant(v)) {
        auto res = this->createNode(PFGNodeType::Constant,
                                    getNameOrAsOperand(v), "", nullptr);
        return res;
    }
    auto inst = dyn_cast<Instruction>(v);

    if (isa<BinaryOperator>(inst) || isa<CallInst>(inst) ||
        isa<CmpInst>(inst)) {
        auto res = this->createNode(PFGNodeType::Expression,
                                    getNameOrAsOperand(v), "", inst);
        return res;
    }

    // Match component first.
    if (isComponentVariable(inst) && isRead(inst)) {
        auto inst_str = inst->getName().str();
        auto gep = dyn_cast<Instruction>(inst->getOperand(0));
        return this->getNodes(resolveSignalOrValue(gep))[0];
    }

    // We don't care the case that the argument is an array.
    if (isSignal(inst) && isRead(inst)) {
        auto inst_str = inst->getName().str();
        auto gep = dyn_cast<Instruction>(inst->getOperand(0));
        return this->getNodes(resolveSignalOrValue(gep))[0];
    }

    if (isVariable(inst) && isRead(inst)) {
        auto expr = resolveVariable(inst);
        auto res = this->locateSourceNode(expr);
        return res;
    }

    if (isa<AllocaInst>(inst)) {
        auto alloca_inst = dyn_cast<AllocaInst>(inst);
        if (alloca_inst->getAllocatedType()->isArrayTy()) {
            return nullptr;
        }
    }

    std::cerr << "Uncovered case in locateSourceNode.";
    printLLVMValue(v);
    assert(false);
}

PFGNode *PFGraph::locateDestinationNode(Value *v) {
    auto inst = dyn_cast<Instruction>(v);
    if (isVariable(inst)) {
        return nullptr;
    }
    if (isSignal(inst)) {
        return this->getNodes(resolveSignalOrValue(inst))[0];
    }
    if (isComponentVariable(inst)) {
        return this->getNodes(resolveSignalOrValue(inst))[0];
    }

    std::cerr << "Uncovered case in locateDestinationNode.";
    printLLVMValue(v);
    assert(false);
}

PFGraph::PFGraph(GraphMap global_graphs, Collector *collector) {
    NameSet hacking_templates = {
        // Problematic templates
        "EllipticCurveScalarMultiplyFp2",
        "EllipticCurveScalarMultiplyUnequalFp2",
        "EllipticCurveScalarMultiply",
        "EllipticCurveScalarMultiplyUnequal",
        "MillerLoop",
        "MillerLoopFp2",
        "MillerLoopFp2Two",
        "OptSimpleSWU2",
        "MultiAND",

        // It is safe but it's hard to implement
        "PoseidonEx",

        // They cost too much time to build.
        "Sha256compression",
        "Sha256",
        "Sha256Bytes",
        "ECDSAPrivToPub",
    };
    auto template_name = collector->template_name;
    this->is_hacking = hacking_templates.count(template_name);
    this->is_instantiation = collector->is_instantiation;
    this->global_graphs = global_graphs;
    this->collector = collector;
    this->nodes = NodeMap();
    this->edges = EdgeMap();
    this->node_idxes = NameIdx();

    // Build graph
    this->build();

    this->uf = new UnionFind(this->nodes.size());

    this->nodeFlowsTo = NameSetMap();
}

NodeVec PFGraph::getNodes(PFGNodeType type, std::string signal_name) {
    NodeVec nodes = NodeVec();
    auto array_shape = this->getArrayShape(signal_name);
    auto array_index = ArrayShape();
    ArrayShapePair ap = {array_shape, array_index};
    auto node_names = this->collector->genAllNames(signal_name, ap);
    for (auto n : node_names) {
        nodes.push_back(this->getNode(type, n));
    }
    return nodes;
}

void PFGraph::buildSelfSignalNodes(NameSet signal_names, PFGNodeType node_ty) {
    for (auto signal_name : signal_names) {
        auto array_shape = this->getArrayShape(signal_name);
        auto array_index = ArrayShape();
        ArrayShapePair ap = {array_shape, array_index};
        auto node_names = this->collector->genAllNames(signal_name, ap);
        for (auto n : node_names) {
            this->createNode(node_ty, n, signal_name, nullptr);
        }
    }
}

void PFGraph::buildCompSignalNodes() {
    for (auto p : this->collector->comp_var2comp_names) {
        // var_name: n2b[0]
        auto var_name = p.first;
        auto comp_name = p.second;
        auto input_signal_names =
            this->collector->getCompInputSignals(comp_name);
        auto output_signal_names =
            this->collector->getCompOutputSignals(comp_name);

        auto sub_graph = this->global_graphs[comp_name];
        // example: n2b_in, prefix is num2bits_
        auto prefix_size = var_name.size() + 1;
        auto i_nodes =
            this->buildCompSignalNodes(sub_graph, var_name, input_signal_names,
                                       PFGNodeType::ComponentInput);
        auto o_nodes =
            this->buildCompSignalNodes(sub_graph, var_name, output_signal_names,
                                       PFGNodeType::ComponentOutput);
        for (auto i : i_nodes) {
            auto i_name = i->getName().substr(prefix_size);
            auto i_hash = i->getHash(PFGNodeType::InputSignal, i_name);
            for (auto i1 : i_nodes) {
                auto i1_name = i1->getName().substr(prefix_size);
                auto i1_hash = i1->getHash(PFGNodeType::InputSignal, i1_name);
                if (i_hash == i1_hash) {
                    continue;
                }
                if (sub_graph->isConstrained(i_hash, i1_hash)) {
                    this->createEdge(PFGEdgeType::Constraint, i, i1);
                }
            }
            for (auto o : o_nodes) {
                auto o_name = o->getName().substr(prefix_size);
                auto o_hash = o->getHash(PFGNodeType::OutputSignal, o_name);
                if (sub_graph->isDepended(i_hash, o_hash)) {
                    this->createEdge(PFGEdgeType::Assignment, i, o);
                }
                if (sub_graph->isConstrained(i_hash, o_hash)) {
                    this->createEdge(PFGEdgeType::Constraint, i, o);
                }
            }
        }
    }
}

NodeVec PFGraph::buildCompSignalNodes(PFGraph *sub_graph,
                                      std::string comp_var_name,
                                      NameSet signal_names,
                                      PFGNodeType node_ty) {
    auto nodes = NodeVec();
    for (auto signal_name : signal_names) {
        auto comp_signal_name = mergeNamePair({comp_var_name, signal_name});
        auto array_shape = sub_graph->getArrayShape(signal_name);
        auto array_index = ArrayShape();
        ArrayShapePair ap = {array_shape, array_index};
        auto node_names = this->collector->genAllNames(comp_signal_name, ap);
        for (auto n : node_names) {
            auto node = this->createNode(node_ty, n, comp_var_name, nullptr);
            nodes.push_back(node);
        }
    }
    return nodes;
}

void PFGraph::build() {
    if (this->is_hacking) {
        return;
    }
    this->buildSelfSignalNodes(this->collector->input_signal_names,
                               PFGNodeType::InputSignal);
    this->buildSelfSignalNodes(this->collector->inter_signal_names,
                               PFGNodeType::IntermediateSignal);
    this->buildSelfSignalNodes(this->collector->output_signal_names,
                               PFGNodeType::OutputSignal);
    this->buildCompSignalNodes();

    for (auto constraint : this->collector->constraints) {
        auto from = constraint->getArgOperand(0);
        auto from_node = this->locateSourceNode(from);
        auto to = constraint->getArgOperand(1);
        auto to_node = this->locateSourceNode(to);
        // Ignore the writing to a variable.
        if (from_node == nullptr || to_node == nullptr) {
            continue;
        }
        this->createEdge(PFGEdgeType::Constraint, from_node, to_node);
    };

    for (auto p : this->collector->assignment_pairs) {
        auto to_node = this->locateDestinationNode(p.second);
        auto from_node = this->locateSourceNode(p.first);
        // Ignore the writing to a variable.
        if (from_node == nullptr || to_node == nullptr) {
            continue;
        }
        this->createEdge(PFGEdgeType::Assignment, from_node, to_node);
    };
}

NodeVec PFGraph::flatExpression(Value *v) {
    if (isArgument(v) || isConstant(v)) {
        return NodeVec();
    }
    auto res = NodeVec();
    auto inst = dyn_cast<Instruction>(v);
    bool uncovered = false;
    if (isRead(inst)) {
        auto d = dyn_cast<Instruction>(inst->getOperand(0));
        if (isVariable(inst)) {
            auto expr = resolveVariable(inst);
            auto expr_str = inst->getName().str();
            auto temp = flatExpression(expr);
            res.insert(res.end(), temp.begin(), temp.end());
        } else {
            auto vs = resolveSignalOrValue(d);
            auto temp = this->getNodes(vs);
            res.insert(res.end(), temp.begin(), temp.end());
        }
    } else if (isa<BinaryOperator>(inst) || isa<CmpInst>(inst)) {
        for (auto &opd : inst->operands()) {
            auto temp = flatExpression(opd);
            res.insert(res.end(), temp.begin(), temp.end());
        }
    } else if (isa<CallInst>(inst)) {
        auto call_inst = dyn_cast<CallInst>(inst);
        auto arg_operands = llvm::make_range(call_inst->op_begin(), call_inst->op_end() - 1);
        for (auto &opd : arg_operands) {
            auto temp = flatExpression(opd);
            res.insert(res.end(), temp.begin(), temp.end());
        }
    } else if (isa<AllocaInst>(inst)) {
        // Do nothing.
    } else {
        uncovered = true;
    }
    if (uncovered) {
        std::cerr << "Uncovered case in flatValueDepends.";
        printLLVMValue(v);
        assert(false);
    }
    return res;
}

NameSet PFGraph::flowsTo(PFGNode *n) {
    if (this->nodeFlowsTo.count(n->getHash())) {
        return this->nodeFlowsTo[n->getHash()];
    }
    NameSet flowsToSet = NameSet();
    for (auto e : n->flowto) {
        if (e->type == PFGEdgeType::Constraint) {
            continue;
        }
        auto to = e->to;
        if (to->isSignal()) {
            auto flowsToSub = this->flowsTo(to);
            flowsToSet =
                makeUnion<NameSet, std::string>(flowsToSet, flowsToSub);
            flowsToSet.insert(to->getHash());
        }
        if (to->type == PFGNodeType::Expression) {
            for (auto e1 : to->flowto) {
                if (e1->type == PFGEdgeType::Constraint) {
                    continue;
                }
                auto to1 = e1->to;
                if (to1->isSignal()) {
                    auto flowsToSub = this->flowsTo(to1);
                    flowsToSet =
                        makeUnion<NameSet, std::string>(flowsToSet, flowsToSub);
                    flowsToSet.insert(to1->getHash());
                }
            }
        }
    }
    this->nodeFlowsTo[n->getHash()] = flowsToSet;
    return flowsToSet;
}

NodeVec PFGraph::getNodes(LLVMValueAccess lv) {
    auto v = lv.first.first;
    if (isArgument(v)) {
        std::string name = "arg" + v->getName().str();
        return {this->getNode(PFGNodeType::Argument, name)};
    }
    if (isConstant(v)) {
        std::string name = getNameOrAsOperand(v);
        return {this->getNode(PFGNodeType::Constant, name)};
    }
    if (!isa<Instruction>(v)) {
        std::cerr << "Error: Uncovered case in getNodes.\n";
        assert(false);
    }
    auto inst = dyn_cast<Instruction>(v);
    auto res = NodeVec();
    if (isSelfSignal(inst)) {
        auto signal_name = extractValueName(inst);
        auto array_shape = this->getArrayShape(signal_name);
        auto array_index = genArrayShape(lv.second);
        ArrayShapePair ap = {array_shape, array_index};
        auto node_names = this->collector->genAllNames(signal_name, ap);
        auto node_ty = lv.first.second;
        for (auto n : node_names) {
            res.push_back(this->getNode(node_ty, n));
        }
        return res;
    } else if (isComponentVariable(inst)) {
        // Compute the variable name without idx.
        // Example: init.n2b.comp -> n2b.
        auto var_name = extractValueName(inst);
        // Get the two access -> [0] and [2].
        CircomValue signal;
        auto access0 = ValueVec();
        auto access1 = ValueVec();
        auto i = 0;
        auto access = lv.second;
        for (auto v : access) {
            if (!isa<GetElementPtrInst>(v)) {
                std::cerr
                    << "Error: Access must be GEP instruction, current is:";
                v->print(errs());
                assert(false);
            }
            auto gep = dyn_cast<GetElementPtrInst>(v);
            if (gep->getSourceElementType()->isStructTy()) {
                access0.insert(access0.begin(), access.begin(),
                               access.begin() + i);
                access1.insert(access1.begin(), access.begin() + i + 1,
                               access.end());
                signal = extractValue(gep).second;
            }
            i++;
        }

        // Compute the component variable's names.
        auto array_shape0 = this->getArrayShape(var_name);
        auto array_index0 = genArrayShape(access0);
        ArrayShapePair ap0 = {array_shape0, array_index0};
        auto comp_var_names = this->collector->genAllNames(var_name, ap0);
        for (auto c : comp_var_names) {
            auto comp_name = this->collector->comp_var2comp_names[c];
            auto sub_graph = this->global_graphs[comp_name];

            // Add signal name
            auto signal_name = splitNamePairMerged(signal.first).second;
            auto node_ty = signal.second;
            NamePair var_signal_p = {c, signal_name};
            auto comp_signal_name = mergeNamePair(var_signal_p);
            auto array_shape1 = sub_graph->getArrayShape(signal_name);
            auto array_index1 = genArrayShape(access1);
            ArrayShapePair ap1 = {array_shape1, array_index1};
            auto node_names =
                this->collector->genAllNames(comp_signal_name, ap1);
            for (auto n : node_names) {
                res.push_back(this->getNode(node_ty, n));
            }
        }
        return res;
    }
    return res;
}

PFGNode *PFGraph::createNode(PFGNodeType type, std::string name,
                             std::string var_name, Instruction *inst) {
    if (type == PFGNodeType::Component || type == PFGNodeType::Variable) {
        std::cerr << "Invalid node type: " << typeEnumToAbbr(type) << "\n";
        assert(false);
    }
    auto hash = PFGNode::getHash(type, name);
    if (this->nodes.count(hash)) {
        return this->nodes[hash];
    }
    auto node = new PFGNode(type, name, var_name, inst);
    this->node_idxes.insert({hash, this->nodes.size()});
    this->nodes.insert({hash, node});
    if (type == PFGNodeType::Expression) {
        auto nodes = flatExpression(inst);
        for (auto n : nodes) {
            this->createEdge(PFGEdgeType::Assignment, n, node);
        }
    }
    return node;
}

PFGEdge *PFGraph::createEdge(PFGEdgeType type, PFGNode *from, PFGNode *to) {
    auto hash = PFGEdge::getHash(type, from, to);
    if (this->edges.count(hash)) {
        return this->edges[hash];
    }
    auto edge = new PFGEdge(type, from, to);
    from->addEdge(edge);
    to->addEdge(edge);
    this->edges.insert({hash, edge});
    return edge;
}

PFGNode *PFGraph::getNode(PFGNodeType type, std::string name) {
    auto hash = PFGNode::getHash(type, name);
    if (this->nodes.count(hash)) {
        return this->nodes[hash];
    }
    std::cerr << "Unknown node: " << hash << "\n";
    assert(false);
}

size_t PFGraph::getNodeIndex(PFGNode *n) {
    auto hash = n->getHash();
    if (this->node_idxes.count(hash)) {
        return this->node_idxes[hash];
    }
    std::cerr << "Unknown node: " << hash << "\n";
    assert(false);
}

std::string PFGraph::getName() { return this->collector->getName(); }

json::Object PFGraph::format() {
    auto obj = json::Object();
    NameVec node_strs = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (n->type == PFGNodeType::Constant) {
            continue;
        }
        node_strs.push_back(n->format());
    }
    obj["Nodes"] = node_strs;

    NameVec edge_strs = NameVec();
    for (auto p : this->edges) {
        auto e = p.second;
        edge_strs.push_back(e->format());
    };
    obj["Edges"] = edge_strs;
    return obj;
}

ArrayShape PFGraph::getArrayShape(std::string var_name) {
    if (!this->collector->array_shapes.count(var_name)) {
        return ArrayShape();
    }
    return this->collector->array_shapes[var_name];
}

void PFGraph::compute() {
    if (this->is_hacking) {
        return;
    }

    // Compute constrained.
    for (auto p : this->edges) {
        auto e = p.second;
        if (e->type == PFGEdgeType::Assignment) {
            continue;
        }
        auto nodes = NodeVec();
        if (e->from->type == PFGNodeType::Expression) {
            for (auto d : e->from->flowfrom) {
                nodes.push_back(d->from);
            }
        } else {
            nodes.push_back(e->from);
        }
        if (e->to->type == PFGNodeType::Expression) {
            for (auto d : e->to->flowfrom) {
                nodes.push_back(d->from);
            }
        } else {
            nodes.push_back(e->to);
        }
        for (size_t i = 0; i < nodes.size() - 1; i++) {
            auto fn = nodes[i];
            auto tn = nodes[i + 1];
            this->uf->merge(this->getNodeIndex(fn), this->getNodeIndex(tn));
        }
    }

    //  Compute data dependency.
    for (auto p : this->nodes) {
        auto n = p.second;
        if (n->isSignal()) {
            this->flowsTo(n);
        }
    }
}

bool PFGraph::isDepended(std::string node_hash_a, std::string node_hash_b) {
    if (this->is_hacking) {
        return true;
    }
    if (!this->nodeFlowsTo.count(node_hash_b)) {
        // No computed yet.
        return false;
    }
    return this->nodeFlowsTo[node_hash_b].count(node_hash_a);
}

bool PFGraph::isConstrained(std::string node_hash_a, std::string node_hash_b) {
    if (this->is_hacking) {
        return true;
    }
    auto node_a_idx = this->node_idxes[node_hash_a];
    auto node_b_idx = this->node_idxes[node_hash_b];
    return this->uf->find(node_a_idx) == this->uf->find(node_b_idx);
}

bool PFGraph::constrainedByInput(PFGNode *n) {
    for (auto p : this->nodes) {
        auto n1 = p.second;
        if (n1->type != PFGNodeType::InputSignal) {
            continue;
        }
        if (this->isConstrained(n->getHash(), p.first)) {
            return true;
        }
    }
    return false;
}

bool PFGraph::constrainedAsConst(PFGNode *n) {
    for (auto e : n->flowto) {
        if (e->type == PFGEdgeType::Constraint) {
            if (e->to->type == PFGNodeType::Constant) {
                return true;
            }
        }
    }
    for (auto e : n->flowfrom) {
        if (e->type == PFGEdgeType::Constraint) {
            if (e->from->type == PFGNodeType::Constant) {
                return true;
            }
            if (e->from->type == PFGNodeType::IntermediateSignal) {
                if (this->constrainedAsConst(e->from)) {
                    return true;
                }
            }
        }
    }
    return false;
}

bool PFGraph::unconstrainedOutput(PFGNode *n) {
    if (this->is_hacking) {
        return false;
    }
    if (n->type != PFGNodeType::OutputSignal) {
        return false;
    }
    bool constrainedByInput = this->constrainedByInput(n);
    bool constrainedAsConst = this->constrainedAsConst(n);
    return !(constrainedByInput || constrainedAsConst);
}

NameVec PFGraph::detectUnconstrainedOutput() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->unconstrainedOutput(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

bool PFGraph::unconstrainedCompInput(PFGNode *n) {
    if (this->is_hacking) {
        return false;
    }
    if (n->type != PFGNodeType::ComponentInput) {
        return false;
    }
    for (auto e : n->flowfrom) {
        if (e->type == PFGEdgeType::Constraint &&
            e->from->var_name != n->var_name) {
            return false;
        }
    }
    for (auto e : n->flowto) {
        if (e->type == PFGEdgeType::Constraint &&
            e->to->var_name != n->var_name) {
            return false;
        }
        if (e->to->type == PFGNodeType::Expression) {
            auto to = e->to;
            for (auto e1 : to->flowto) {
                if (e1->type == PFGEdgeType::Constraint) {
                    return false;
                }
            }
            for (auto e1 : to->flowfrom) {
                if (e1->type == PFGEdgeType::Constraint) {
                    return false;
                }
            }
        }
    }
    return true;
}

NameVec PFGraph::detectUnconstrainedCompInput() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->unconstrainedCompInput(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

NameVec PFGraph::detectDataflowConstraintDis() {
    auto names = NameVec();
    for (auto p : nodeFlowsTo) {
        auto n_name = p.first;
        auto n_set = p.second;
        for (auto n_name1 : n_set) {
            if (!this->isConstrained(n_name, n_name1)) {
                auto rep =
                    n_name + " flows to " + n_name1 + " but not constrains";
                names.push_back(rep);
            }
        }
    }
    return names;
}

bool PFGraph::isCheckingSignal(PFGNode *n) {
    for (auto e : n->flowto) {
        if (e->type == PFGEdgeType::Assignment) {
            return true;
        }
    }
    return false;
}

bool PFGraph::unusedCompOutput(PFGNode *n) {
    if (this->is_hacking) {
        return false;
    }
    if (n->type != PFGNodeType::ComponentOutput) {
        return false;
    }
    auto var_signal_p = splitNamePairMerged(n->getName());
    auto var_i_name = var_signal_p.first;
    auto signal_name = var_signal_p.second;
    auto comp_name = this->collector->comp_var2comp_names[var_i_name];
    auto sub_graph = this->global_graphs[comp_name];
    if (sub_graph->is_hacking) {
        return false;
    }
    auto sub_o_node =
        sub_graph->getNode(PFGNodeType::OutputSignal, signal_name);
    if (sub_graph->isCheckingSignal(sub_o_node)) {
        return false;
    }
    for (auto e : n->flowto) {
        if (e->to->var_name != n->var_name) {
            return false;
        }
    }
    for (auto e : n->flowfrom) {
        if (e->from->var_name != n->var_name) {
            return false;
        }
    }
    return true;
}

NameVec PFGraph::detectUnusedCompOutput() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->unusedCompOutput(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

bool PFGraph::unusedSignal(PFGNode *n) {
    if (this->is_hacking) {
        return false;
    }
    if (n->type == PFGNodeType::ComponentOutput ||
        n->type == PFGNodeType::Constant) {
        return false;
    }
    return (n->flowfrom.size() + n->flowto.size()) == 0;
}

NameVec PFGraph::detectUnusedSignal() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->unusedSignal(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

bool PFGraph::isDenominatorWithSignal(Instruction *inst) {
    if (isa<BinaryOperator>(inst)) {
        auto bin_inst = dyn_cast<BinaryOperator>(inst);
        if (bin_inst->getOpcode() == Instruction::SDiv) {
            if (flatExpression(bin_inst->getOperand(1)).size() > 0) {
                return true;
            };
        };
        for (auto &opd : bin_inst->operands()) {
            if (isa<Instruction>(opd)) {
                auto i = dyn_cast<Instruction>(opd);
                if (this->isDenominatorWithSignal(i)) {
                    return true;
                }
            }
        }
        return false;
    }
    return false;
}

bool PFGraph::isDivideByZeroUnsafe(PFGNode *n) {
    if (n->type != PFGNodeType::Expression) {
        return false;
    }
    if (!this->isDenominatorWithSignal(n->inst)) {
        return false;
    }
    return true;
}

NameVec PFGraph::detectDivideByZeroUnsafe() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->isDivideByZeroUnsafe(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

bool PFGraph::isBranchCondWithSignal(Instruction *inst) {
    if (isa<BinaryOperator>(inst)) {
        auto bin_inst = dyn_cast<BinaryOperator>(inst);
        for (auto &opd : bin_inst->operands()) {
            if (isa<Instruction>(opd)) {
                auto i = dyn_cast<Instruction>(opd);
                if (this->isBranchCondWithSignal(i)) {
                    return true;
                }
            }
        }
    }
    if (isa<CallInst>(inst)) {
        auto call_inst = dyn_cast<CallInst>(inst);
        auto called_func = call_inst->getCalledFunction();
        if (called_func->getName().startswith(fn_switch_prefix)) {
            if (flatExpression(call_inst->getArgOperand(0)).size() > 0) {
                return true;
            };
        }
    }
    return false;
}

// bool PFGraph::isMultipleExpression(Instruction *inst) {
//     if (isa<BinaryOperator>(inst)) {
//         auto bin_inst = dyn_cast<BinaryOperator>(inst);
//         if (bin_inst->getOpcode() == Instruction::Mul) {
//             return true;
//         };
//         for (auto &opd : bin_inst->operands()) {
//             if (isa<Instruction>(opd)) {
//                 auto i = dyn_cast<Instruction>(opd);
//                 if (this->isSensitiveExpression(i)) {
//                     return true;
//                 }
//             }
//         }
//     }
//     return false;
// }

bool PFGraph::isBranchCondWithSignal(PFGNode *n) {
    if (n->type != PFGNodeType::Expression) {
        return false;
    }
    if (!this->isBranchCondWithSignal(n->inst)) {
        return false;
    }
    return true;
}

NameVec PFGraph::detectNondeterministicDataflow() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->isBranchCondWithSignal(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

NameVec PFGraph::detectTypeMismatch() {
    // Mock
    NameSet num2bits_required = {
        "LessThan", "LessEqThan", "GreaterThan", "GreaterEqThan", "BigLessThan",
    };

    // Mock
    NameSet num2bits_like = {
        "Num2Bits",
        "Num2Bits_strict",
        "RangeProof",
        "MultiRangeProof",
        "RangeCheck2D",
    };

    if (this->is_hacking) {
        return NameVec();
    }

    NameVec results = NameVec();

    for (auto p : this->nodes) {
        auto n = p.second;
        if (n->type != PFGNodeType::ComponentInput) {
            continue;
        }

        auto comp_var_name = n->var_name;
        auto comp_name = this->collector->comp_var2comp_names[comp_var_name];
        auto template_name = normalizeComponentName(comp_name);

        if (!num2bits_required.count(template_name)) {
            continue;
        }

        auto input_nodes = NodeVec();
        for (auto e : n->flowfrom) {
            if (e->type == PFGEdgeType::Constraint) {
                continue;
            }
            if (e->from->isSignal()) {
                input_nodes.push_back(e->from);
            }
            if (e->from->type == PFGNodeType::Expression) {
                for (auto e1 : e->from->flowfrom) {
                    if (e1->type == PFGEdgeType::Assignment &&
                        e1->from->isSignal()) {
                        input_nodes.push_back(e1->from);
                    }
                }
            }
        }

        for (auto n1 : input_nodes) {
            auto isChecked = false;
            auto flowsTo = this->flowsTo(n1);
            for (auto n2_hash : flowsTo) {
                auto n2 = this->nodes[n2_hash];
                if (n2->type != PFGNodeType::ComponentInput) {
                    continue;
                }
                auto comp_var_name = n2->var_name;
                auto comp_name =
                    this->collector->comp_var2comp_names[comp_var_name];
                auto template_name = normalizeComponentName(comp_name);
                if (num2bits_like.count(template_name)) {
                    isChecked = true;
                }
            }
            if (!isChecked) {
                auto output = "Node: " + n->getName() + "'s input " +
                              n1->getName() + " is not checked";
                results.push_back(output);
            }
        }
    }
    return results;
}

bool PFGraph::isTrivialInstruction(Value *v) {
    if (!isa<Instruction>(v)) {
        return true;
    }
    auto inst = dyn_cast<Instruction>(v);
    if (isa<CallInst>(inst)) {
        return false;
    }
    if (isa<CmpInst>(inst)) {
        return false;
    }
    if (isa<UnaryOperator>(inst)) {
        return true;
    }
    if (isa<AllocaInst>(inst) || isa<LoadInst>(inst)) {
        return true;
    }
    if (isa<BinaryOperator>(inst)) {
        auto bin_inst = dyn_cast<BinaryOperator>(inst);
        if (bin_inst->getOpcode() == Instruction::Add ||
            bin_inst->getOpcode() == Instruction::Sub ||
            bin_inst->getOpcode() == Instruction::Mul) {
            auto is_trivial = true;
            for (auto &opd : bin_inst->operands()) {
                is_trivial |= this->isTrivialInstruction(opd);
            }
            return is_trivial;
        } else if (bin_inst->getOpcode() == Instruction::SDiv) {
            auto dem = bin_inst->getOperand(1);
            if (!isa<Instruction>(dem)) {
                return this->isTrivialInstruction(bin_inst->getOperand(0));
            }
            return false;
        } else {
            return false;
        }
    }
    return false;
}

bool PFGraph::isRewritableAssignment(PFGEdge *e) {
    if (e->type != PFGEdgeType::Assignment) {
        return false;
    }
    auto from = e->from;
    auto to = e->to;
    for (auto e1 : from->flowto) {
        if (e1->to == to && e1->type == PFGEdgeType::Constraint) {
            return false;
        }
    }
    for (auto e1 : from->flowfrom) {
        if (e1->from == to && e1->type == PFGEdgeType::Constraint) {
            return false;
        }
    }
    if (to->type == PFGNodeType::Expression) {
        return false;
    }
    // Now, to node is a signal.
    if (from->type != PFGNodeType::Expression) {
        return true;
    }
    // Now, from node is an expression.
    return this->isTrivialInstruction(from->inst);
}

NameVec PFGraph::detectAssignmentMisuse() {
    NameVec results = NameVec();
    for (auto p : this->edges) {
        auto e = p.second;
        if (this->isRewritableAssignment(e)) {
            results.push_back(e->getName());
        }
    }
    return results;
}

NameVec PFGraph::detectNonuniformConstraint() {
    if (this->is_hacking) {
        return NameVec();
    }
    auto results = NameVec();
    std::vector<std::pair<PFGNodeType, std::string>> ty_name_pairs = {};
    for (auto n : this->collector->input_signal_names) {
        ty_name_pairs.push_back({PFGNodeType::InputSignal, n});
    }
    for (auto n : this->collector->inter_signal_names) {
        ty_name_pairs.push_back({PFGNodeType::IntermediateSignal, n});
    }
    for (auto n : this->collector->output_signal_names) {
        ty_name_pairs.push_back({PFGNodeType::OutputSignal, n});
    }
    for (auto p : ty_name_pairs) {
        auto nodes = this->getNodes(p.first, p.second);
        auto constrain_nums = std::vector<size_t>();
        auto first_constrain_num = 0;
        auto keep_the_same = true;
        for (auto n : nodes) {
            auto constrain_num = 0;
            for (auto e : n->flowto) {
                if (e->type == PFGEdgeType::Constraint) {
                    constrain_num += 1;
                }
                if (e->to->type == PFGNodeType::Expression) {
                    for (auto e1 : e->to->flowto) {
                        if (e1->type == PFGEdgeType::Constraint) {
                            constrain_num += 1;
                        }
                    }
                    for (auto e1 : e->to->flowfrom) {
                        if (e1->type == PFGEdgeType::Constraint) {
                            constrain_num += 1;
                        }
                    }
                }
            }
            for (auto e : n->flowfrom) {
                if (e->type == PFGEdgeType::Constraint) {
                    constrain_num += 1;
                }
            }
            if (first_constrain_num == 0) {
                first_constrain_num = constrain_num;
            }
            if (constrain_num != first_constrain_num) {
                keep_the_same = false;
            }
            constrain_nums.push_back(constrain_num);
        }
        if (!keep_the_same) {
            std::stringstream ss;
            ss << p.second + ", Size: " << nodes.size() << ", Nums: ";
            for (auto i : constrain_nums) {
                ss << i << ",";
            }
            results.push_back(ss.str());
        }
    }
    return results;
}

bool PFGraph::isQuadricExpression(PFGNode *n) {
    if (n->type != PFGNodeType::Expression) {
        return false;
    }
    auto mul_insts = std::vector<BinaryOperator *>();
    InstructionVec todo_insts = {n->inst};
    while (todo_insts.size() > 0) {
        auto inst = todo_insts.back();
        todo_insts.pop_back();
        if (isa<BinaryOperator>(inst)) {
            auto bin_inst = dyn_cast<BinaryOperator>(inst);
            if (bin_inst->getOpcode() == Instruction::Mul) {
                mul_insts.push_back(bin_inst);
            };
            for (auto &opd : bin_inst->operands()) {
                if (isa<Instruction>(opd)) {
                    auto i = dyn_cast<Instruction>(opd);
                    todo_insts.push_back(i);
                }
            }
        }
    }
    for (auto i : mul_insts) {
        auto l_nodes = this->flatExpression(i->getOperand(0));
        auto r_nodes = this->flatExpression(i->getOperand(1));
        if (l_nodes.size() != 0 && r_nodes.size() != 0) {
            return true;
        }
    }
    return false;
}

bool PFGraph::isUnconstrainedSignalWithoutQC(PFGNode *n) {
    if (n->type == PFGNodeType::Constant ||
        n->type == PFGNodeType::Expression) {
        return false;
    }
    for (auto e : n->flowfrom) {
        if (e->type == PFGEdgeType::Constraint) {
            if (!this->isQuadricExpression(e->from)) {
                return false;
            }
        }
    }
    for (auto e1 : n->flowto) {
        if (e1->type == PFGEdgeType::Constraint) {
            if (!this->isQuadricExpression(e1->to)) {
                return false;
            }
        } else if (!this->isQuadricExpression(e1->to)) {
            for (auto e2 : e1->to->flowfrom) {
                if (e2->type == PFGEdgeType::Constraint) {
                    return false;
                }
            }
            for (auto e2 : e1->to->flowto) {
                if (e2->type == PFGEdgeType::Constraint) {
                    return false;
                }
            }
        }
    }
    return true;
}

NameVec PFGraph::detectUnconstrainedSignalWithoutQC() {
    NameVec results = NameVec();
    for (auto p : this->nodes) {
        auto n = p.second;
        if (this->isUnconstrainedSignalWithoutQC(n)) {
            results.push_back(n->getName());
        }
    }
    return results;
}

GraphVec initDetectedGraphs(Module &M, bool compute, bool only_main) {
    auto graphs = GraphVec();
    auto global_graphs = GraphMap();
    auto ordered_functions = sortFunctions(&M);
    auto ordered_collectors = sortCollectors(ordered_functions);
    auto main_comp = extractMainComp(&M);
    for (auto c : ordered_collectors) {
        auto graph = new PFGraph(global_graphs, c);
        global_graphs.insert({graph->getName(), graph});
        if (compute) {
            graph->compute();
        }
        if (main_comp != "" && main_comp != c->getName() && only_main) {
            continue;
        }
        graphs.push_back(graph);
    }
    return graphs;
}
