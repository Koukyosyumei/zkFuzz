#include "utils_basicinfo.hpp"

ValueTypeEnum abbrToTypeEnum(std::string s) {
    for (auto ve : VariableTypeAll) {
        if (typeEnumToAbbr(ve) == s) {
            return ve;
        }
    }
    std::cerr << "Invalid type enum string! " << s << "\n";
    assert(false);
}

std::string typeEnumToAbbr(ValueTypeEnum ve) {
    switch (ve) {
        case ValueTypeEnum::InputSignal:
            return "input";
        case ValueTypeEnum::IntermediateSignal:
            return "inter";
        case ValueTypeEnum::OutputSignal:
            return "output";
        case ValueTypeEnum::ComponentInput:
            return "compinput";
        case ValueTypeEnum::ComponentOutput:
            return "compoutput";
        case ValueTypeEnum::Argument:
            return "arg";
        case ValueTypeEnum::Constant:
            return "const";
        case ValueTypeEnum::Expression:
            return "expr";
        case ValueTypeEnum::Component:
            return "comp";
        case ValueTypeEnum::Variable:
            return "var";
    }
}

bool isInstantiation(Module *M) {
    return M->getGlobalVariable("is_instantiation") != nullptr;
}

std::string extractMainComp(Module *M) {
    for (auto &g : M->getGlobalList()) {
        if (g.getName().startswith("main_comp=")) {
            return g.getName().ltrim("main_comp=").str();
        }
    }
    return "";
}

bool isTemplateInitFunc(Function *F) {
    return F->getName().startswith(fn_template_prefix);
}

bool isTemplateBuildFunc(Function *F) {
    return F->getName().startswith(fn_build_prefix);
}

bool isPowiFunc(Function *F) { return F->getName().startswith(fn_powi_prefix); }

bool isSwitchFunc(Function *F) {
    return F->getName().startswith(fn_switch_prefix);
}

bool isEntry(BasicBlock *bb) { return bb->getName().startswith("entry"); }

bool isExit(BasicBlock *bb) { return bb->getName().startswith("exit"); }

bool locatesAtEntry(Instruction *inst) { return isEntry(inst->getParent()); }

bool locatesAtExit(Instruction *inst) { return isExit(inst->getParent()); }

bool isComponent(Instruction *inst) {
    if (isa<Component>(inst)) {
        Component *called_inst = dyn_cast<Component>(inst);
        Function *F = called_inst->getCalledFunction();
        return F->getName().startswith(fn_build_prefix);
    } else {
        return false;
    }
}

bool isConstraint(Instruction *inst) {
    if (isa<ConstraintInstance>(inst)) {
        ConstraintInstance *called_inst = dyn_cast<ConstraintInstance>(inst);
        Function *F = called_inst->getCalledFunction();
        return F->getName().startswith(fn_constraint_prefix);
    } else {
        return false;
    }
}

bool isInlineArray(Instruction *inst) {
    return inst->getName().startswith(const_inline_array);
}

bool isAssignment(Instruction *inst) {
    if (isa<StoreInst>(inst)) {
        auto store_inst = dyn_cast<StoreInst>(inst);
        if (store_inst->getOperand(0)->getType()->isPointerTy()) {
            auto ptr =
                dyn_cast<PointerType>(store_inst->getOperand(0)->getType());
            if (ptr->getElementType()->isStructTy()) {
                return false;
            }
        }
        return isCircomFormatValue(inst->getOperand(1));
    };
    if (isa<CallInst>(inst)) {
        auto called_func = dyn_cast<CallInst>(inst)->getCalledFunction();
        return called_func->getName().startswith("llvm.memcpy");
    };
    return false;
}

bool isInitial(Instruction *inst) {
    return inst->getName().contains("initial.");
}

bool isRead(Instruction *inst) { return inst->getName().contains("read."); }

bool isWrite(Instruction *inst) { return inst->getName().contains("write."); }

bool isCircomFormatValue(Value *v) {
    auto name = v->getName().str();
    auto res = stringSplit(name, ".", 2);
    return res.size() == 3;
}

bool isInputSignal(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".input");
}

bool isInterSignal(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".inter");
}

bool isOutputSignal(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".output");
}

bool isComponentInput(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".compinput");
}

bool isComponentOutput(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".compoutput");
}

bool isVariable(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".var");
}

bool isArgument(Value *v) { return isa<Argument>(v); }

bool isConstant(Value *v) { return isa<Constant>(v); }

bool isComponentVariable(Instruction *inst) {
    auto origin_name = inst->getName().str();
    auto name = removeIdxInLLVMName(origin_name);
    return endsWith(name, ".comp");
}

bool isSelfSignal(Instruction *inst) {
    return isInputSignal(inst) || isInterSignal(inst) || isOutputSignal(inst);
}

bool isCompSignal(Instruction *inst) {
    return isComponentInput(inst) || isComponentOutput(inst);
}

bool isSignal(Instruction *inst) {
    return isSelfSignal(inst) || isCompSignal(inst);
}

bool isReadSignal(Instruction *inst) { return isSignal(inst) && isRead(inst); }

bool isWriteSignal(Instruction *inst) {
    return isSignal(inst) || isWrite(inst);
}

std::pair<std::string, CircomValue> extractValue(Value *v) {
    // clang-format off
    // %read.in.input = xxx
    // %write.out.output = xxx
    // %initial.out.output = xxx
    // %gep.abc_in.compoutput = xxx
    // clang-format on
    auto name = v->getName().str();
    auto res = stringSplit(name, ".", 2);
    if (res.size() != 3) {
        std::cerr << "Invalid circom value:" << name << "\n";
        assert(false);
    }
    auto op = res[0];
    auto value_name = res[1];
    auto ty_enum_str = res[2];
    ty_enum_str = removeIdxInLLVMName(ty_enum_str);
    auto ty_enum = abbrToTypeEnum(ty_enum_str);
    CircomValue res_t = {value_name, ty_enum};
    std::pair<std::string, CircomValue> p = {op, res_t};
    return p;
}

std::string extractComponentName(Function *F) {
    // clang-format off
    // @fn_template_init_num2bitsneg
    // clang-format on
    std::string comp_name = F->getName().str();
    if (comp_name.rfind(fn_template_prefix, 0) == 0) {
        comp_name.replace(0, fn_template_prefix.length(), "");
    }
    if (comp_name.rfind(fn_build_prefix, 0) == 0) {
        comp_name.replace(0, fn_build_prefix.length(), "");
    }
    return comp_name;
}

std::string extractValueName(Value *v) { return extractValue(v).second.first; }

NamePair extractCompSignal(Value *v) {
    // clang-format off
    // %read.comp_out.compoutput = xxx
    // clang-format on
    auto res = extractValueName(v);
    auto p = splitNamePairMerged(res);
    return p;
}

std::string mergeNamePair(NamePair p) { return p.first + "|" + p.second; }

NamePair splitNamePairMerged(std::string s) {
    auto res = stringSplit(s, "|", INT_MAX);
    if (res.size() != 2) {
        std::cerr << "Uncovered case in splitNamePairMerged." << s << "\n";
        assert(false);
    };
    NamePair p = {res[0], res[1]};
    return p;
}

std::string normalizeComponentName(std::string s) {
    // clang-format off
    // bigsigma@ra=2@rb=13@rc=22 -> bigsigma
    // clang-format on
    return stringSplit(s, "@", INT_MAX)[0];
}

Assignment resolveAssignment(Instruction *inst) {
    Value *source;
    Value *destination;
    if (isa<StoreInst>(inst)) {
        source = inst->getOperand(0);
        destination = inst->getOperand(1);
    } else if (isa<CallInst>(inst)) {
        auto write_from_inst = dyn_cast<BitCastInst>(inst->getOperand(1));
        auto write_to_inst = dyn_cast<BitCastInst>(inst->getOperand(0));
        source = write_from_inst->getOperand(0);
        destination = write_to_inst->getOperand(0);
    } else {
        std::cerr << "Instruction should be an assignment, current is:";
        inst->print(errs());
        std::cerr << "\n";
        assert(false);
    }
    Assignment res = {source, dyn_cast<Instruction>(destination)};
    return res;
}

LLVMValueAccess resolveSignalOrValue(Value *v) {
    auto inst = dyn_cast<Instruction>(v);
    auto access = ValueVec();
    auto cur = inst;
    while (!isInitial(cur)) {
        if (!isa<GetElementPtrInst>(cur)) {
            std::cerr << "Error: This instruction is not a gep instruction.";
            printLLVMValue(cur);
            assert(false);
        }
        auto gep = dyn_cast<GetElementPtrInst>(cur);
        access.insert(access.begin(), gep);
        auto ptr = gep->getOperand(0);
        if (isa<BitCastInst>(ptr)) {
            auto cast_inst = dyn_cast<BitCastInst>(ptr);
            ptr = cast_inst->getOperand(0);
        }
        if (isa<LoadInst>(ptr)) {
            auto load = dyn_cast<LoadInst>(ptr);
            cur = dyn_cast<Instruction>(load->getOperand(0));
        } else {
            std::cerr << "Error: This instruction is not a load instruction.";
            printLLVMValue(gep->getOperand(0));
            assert(false);
        }
    }
    LLVMValue lv = {cur, extractValue(cur).second.second};
    LLVMValueAccess res = {lv, access};
    return res;
}

Value *resolveVariable(Instruction *inst) {
    auto gep = dyn_cast<Instruction>(inst->getOperand(0));
    auto vs = resolveSignalOrValue(gep);
    auto v_init = vs.first.first;
    auto v_name = extractValueName(v_init);
    auto access = genArrayShape(vs.second);
    // This function doesn't consider that the variable is an array.
    auto res = ValueVec();
    auto prev_inst = inst;
    while (prev_inst->getPrevNode() != nullptr) {
        prev_inst = prev_inst->getPrevNode();
        if (isAssignment(prev_inst)) {
            auto p = resolveAssignment(prev_inst);
            auto pv_name = extractValueName(p.second);
            if (pv_name == v_name) {
                return p.first;
            }
        }
    }
}

Collector::Collector(Function *F) {
    if (isTemplateInitFunc(F)) {
        this->F = F;
    } else {
        std::cerr << "Error: This function isn't template function!";
    }
    this->is_instantiation = isInstantiation(F->getParent());
    this->comp_name = extractComponentName(F);
    this->template_name = normalizeComponentName(this->comp_name);
    this->constraints = ConstraintVec();
    this->assignments = InstructionVec();
    this->assignment_pairs = AssignmentVec();

    this->input_signals = ValueAllocaVec();
    this->inter_signals = ValueAllocaVec();
    this->output_signals = ValueAllocaVec();
    this->variables = ValueAllocaVec();
    this->component_variables = ValueAllocaVec();

    this->component_inputs = SignalGEPVec();
    this->component_outputs = SignalGEPVec();

    this->components = ComponentVec();

    this->argument_names = NameSet();
    this->input_signal_names = NameSet();
    this->inter_signal_names = NameSet();
    this->output_signal_names = NameSet();
    this->variable_names = NameSet();

    this->comp_var2comp_names = NameMap();

    this->component_input_signal_names = NameSetMap();
    this->component_output_signal_names = NameSetMap();

    this->array_shapes = ArrayShapeMap();

    for (auto &arg : F->args()) {
        this->argument_names.insert(arg.getName().str());
    }

    for (auto &bb : F->getBasicBlockList()) {
        for (auto &inst : bb.getInstList()) {
            if (isArrayShapeDefinedInst(&inst)) {
                auto called_inst = dyn_cast<ArrayShapeDefinition>(&inst);
                auto p = extractArrayShapeDefine(called_inst);
                // Inline array
                if (!isCircomFormatValue(p.first)) {
                    continue;
                }
                auto var_name = extractValueName(p.first);
                auto array_shape = LLVMtoConcreteArrayShape(p.second);
                this->array_shapes.insert({var_name, array_shape});
            }
        }
    }

    for (auto &bb : F->getBasicBlockList()) {
        for (auto &inst : bb.getInstList()) {
            if (isInputSignal(&inst) && isInitial(&inst)) {
                ValueAlloca *signal = dyn_cast<ValueAlloca>(&inst);
                this->input_signals.push_back(signal);
                this->input_signal_names.insert(extractValueName(signal));
            }
            if (isInterSignal(&inst) && isInitial(&inst)) {
                ValueAlloca *signal = dyn_cast<ValueAlloca>(&inst);
                this->inter_signals.push_back(signal);
                this->inter_signal_names.insert(extractValueName(signal));
            }
            if (isOutputSignal(&inst) && isInitial(&inst)) {
                ValueAlloca *signal = dyn_cast<ValueAlloca>(&inst);
                this->output_signals.push_back(signal);
                this->output_signal_names.insert(extractValueName(signal));
            }
            if (isVariable(&inst) && isInitial(&inst)) {
                ValueAlloca *variable = dyn_cast<ValueAlloca>(&inst);
                this->variables.push_back(variable);
                this->variable_names.insert(extractValueName(variable));
            }
            if (isComponentInput(&inst)) {
                SignalGEP *s = dyn_cast<SignalGEP>(&inst);
                this->component_inputs.push_back(s);
            }
            if (isComponentOutput(&inst)) {
                SignalGEP *s = dyn_cast<SignalGEP>(&inst);
                this->component_outputs.push_back(s);
            }
            if (isConstraint(&inst)) {
                ConstraintInstance *constraint =
                    dyn_cast<ConstraintInstance>(&inst);
                this->constraints.push_back(constraint);
            }
            if (isAssignment(&inst)) {
                if (isEntry(&bb) || isExit(&bb)) {
                    continue;
                }
                this->assignments.push_back(&inst);
                this->assignment_pairs.push_back(resolveAssignment(&inst));
            }
            if (isComponent(&inst)) {
                Component *component = dyn_cast<Component>(&inst);
                this->components.push_back(component);
                auto comp_name =
                    extractComponentName(component->getCalledFunction());
                auto store_inst = dyn_cast<StoreInst>(*inst.user_begin());
                auto comp_var = resolveSignalOrValue(store_inst->getOperand(1));
                auto comp_init = comp_var.first.first;
                auto var_name = extractValueName(comp_init);
                auto array_shape = this->getArrayShape(var_name);
                auto array_index = genArrayShape(comp_var.second);
                ArrayShapePair ap = {array_shape, array_index};
                auto comp_var_name = this->genAllNames(var_name, ap)[0];
                this->comp_var2comp_names.insert({comp_var_name, comp_name});
            }
        }
    }
}

NameVec Collector::genAllNames(std::string name, ArrayShapePair ap) {
    if (ap.first.size() == 0 || !this->is_instantiation) {
        return {name};
    }
    if (ap.first.size() < ap.second.size()) {
        std::cerr << "Error: Array size is less than the index";
        assert(false);
    } else if (ap.first.size() == ap.second.size()) {
        auto index = computeIndex(ap);
        return {this->genNameWithIdx(name, index)};
    } else {
        auto names = NameVec();
        ap.second.insert(ap.second.end(), ap.first.size() - ap.second.size(),
                         0);
        auto start = computeIndex(ap);
        auto end = getArraySize(ap.first);
        for (auto i = start; i < end; i++) {
            names.push_back(this->genNameWithIdx(name, i));
        }
        return names;
    }
}

std::string Collector::genNameWithIdx(std::string var_name, uint64_t index) {
    return var_name + "[" + std::to_string(index) + "]";
}

std::string Collector::getName() { return this->comp_name; }

json::Object Collector::format() {
    auto obj = json::Object();
    obj["Input Signals"] = formatNameSet(this->input_signal_names);
    obj["Inter Signals"] = formatNameSet(this->inter_signal_names);
    obj["Output Signals"] = formatNameSet(this->output_signal_names);
    obj["Variables"] = formatNameSet(this->variable_names);
    obj["Component Input Signals"] =
        formatNameSet(this->flat_component_input_signal_names);
    obj["Component Output Signals"] =
        formatNameSet(this->flat_component_output_signal_names);
    return obj;
}

ArrayShape Collector::getArrayShape(std::string s) {
    return this->array_shapes[s];
}

NameSet Collector::getCompInputSignals(std::string comp_name) {
    if (!this->component_input_signal_names.count(comp_name)) {
        this->component_input_signal_names.insert({comp_name, NameSet()});
    }
    return this->component_input_signal_names[comp_name];
}

NameSet Collector::getCompOutputSignals(std::string comp_name) {
    if (!this->component_output_signal_names.count(comp_name)) {
        this->component_output_signal_names.insert({comp_name, NameSet()});
    }
    return this->component_output_signal_names[comp_name];
}

void Collector::addCompSignals(CollectorMap global_c) {
    for (auto p : this->comp_var2comp_names) {
        auto comp_name = p.second;
        auto sub_graph = global_c[p.second];
        this->component_input_signal_names[comp_name] =
            sub_graph->input_signal_names;
        this->component_output_signal_names[comp_name] =
            sub_graph->output_signal_names;
    }
    this->flat_component_input_signal_names = NameSet();
    this->flat_component_output_signal_names = NameSet();

    for (auto kv : this->component_input_signal_names) {
        for (auto s : kv.second) {
            NamePair p = {kv.first, s};
            this->flat_component_input_signal_names.insert(mergeNamePair(p));
        }
    }

    for (auto kv : this->component_output_signal_names) {
        for (auto s : kv.second) {
            NamePair p = {kv.first, s};
            this->flat_component_output_signal_names.insert(mergeNamePair(p));
        }
    }
}

std::vector<Collector *> sortCollectors(FunctionVec funcs) {
    auto collectors = std::vector<Collector *>();
    for (auto F : funcs) {
        if (!isTemplateInitFunc(F)) {
            continue;
        }
        collectors.push_back(new Collector(F));
    }
    auto resolved_names = std::unordered_set<std::string>();
    auto ordered_collectors = std::vector<Collector *>();
    auto size = collectors.size();
    while (ordered_collectors.size() < size) {
        auto last = collectors.back();
        collectors.pop_back();
        bool sat = true;
        for (auto p : last->comp_var2comp_names) {
            auto c = p.second;
            sat = sat && (resolved_names.count(c) || last->getName() == c);
        }
        if (sat) {
            resolved_names.insert(last->getName());
            ordered_collectors.push_back(last);
        } else {
            collectors.insert(collectors.begin(), last);
        }
    }

    auto global_collectors = CollectorMap();
    for (auto c : ordered_collectors) {
        global_collectors.insert({c->getName(), c});
        c->addCompSignals(global_collectors);
    }

    return ordered_collectors;
}
