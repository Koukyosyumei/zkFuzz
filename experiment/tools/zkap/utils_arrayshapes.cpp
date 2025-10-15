#include "utils_arrayshapes.hpp"

bool isArrayPtrTy(Type* ty) {
    if (!isa<PointerType>(ty)) {
        return false;
    }
    auto ptr_ty = dyn_cast<PointerType>(ty);
    return ptr_ty->getElementType()->isArrayTy();
}

bool isArrayReturnFunction(Function* F) {
    auto ret_ty = F->getReturnType();
    return isArrayPtrTy(ret_ty);
}

bool isArrayShapeDefinedInst(Instruction* inst) {
    if (!isa<ArrayShapeDefinition>(inst)) {
        return false;
    }
    auto called_inst = dyn_cast<ArrayShapeDefinition>(inst);
    return called_inst->getCalledFunction()->getName().startswith(
        array_shape_mark);
}

std::pair<Value*, LLVMArrayShape> extractArrayShapeDefine(
    ArrayShapeDefinition* inst) {
    auto ptr = inst->getArgOperand(0);
    auto ptr_cast_inst = dyn_cast<Instruction>(ptr);
    auto array_val = ptr_cast_inst->getOperand(0);
    auto shape = LLVMArrayShape();
    uint num_arg_operands = inst->getNumOperands() - inst->getNumTotalBundleOperands() - 1;
    for (uint i = 0; i < num_arg_operands; i++) {
        if (i == 0) {
            continue;
        }
        auto val = inst->getArgOperand(i);
        shape.push_back(val);
    }
    return {array_val, shape};
}

void printShape(LLVMArrayShape shape) {
    for (auto& dim : shape) {
        dim->print(errs());
        std::cerr << "; ";
    }
}

ArrayShape genArrayShape(ValueVec access) {
    auto res = LLVMArrayShape();
    for (auto v : access) {
        if (!isa<GetElementPtrInst>(v)) {
            std::cerr << "Error: Access must be GEP instruction, current is:";
            v->print(errs());
            assert(false);
        }
        auto gep = dyn_cast<GetElementPtrInst>(v);
        if (gep->getSourceElementType()->isArrayTy()) {
            auto indexes = gep->operands();
            res.insert(res.begin(), indexes.begin() + 2, indexes.end());
        } else {
            std::cerr
                << "Error: Source of an array shape access must be arrayTy, "
                   "current is:";
            v->print(errs());
            assert(false);
        }
    }
    return LLVMtoConcreteArrayShape(res);
}

ArrayShape LLVMtoConcreteArrayShape(LLVMArrayShape s) {
    auto res = ArrayShape();
    for (auto& v : s) {
        if (isa<ConstantInt>(v)) {
            auto c = dyn_cast<ConstantInt>(v);
            res.push_back(c->getZExtValue());
        } else {
            std::cerr << "Error: Uncovered case in LLVMtoConcreteArrayShape.\n";
            v->print(errs());
            std::cerr << "\n";
            assert(false);
        }
    }
    return res;
}

uint64_t getArraySize(ArrayShape s) {
    if (s.size() == 0) {
        return 0;
    }
    auto res = 1;
    for (auto i: s) {
        res *= i;
    }
    return res;
}

uint64_t computeIndex(ArrayShapePair ap) {
    auto s = ap.first;
    auto p = ap.second;
    assert(s.size() == p.size());
    uint64_t res = 0;
    for (size_t i = 0; i < s.size(); i++) {
        auto start = s.begin() + i + 1;
        auto end = s.end();
        uint64_t acc = std::accumulate(start, end, 1, std::multiplies<uint64_t>());
        res += p[i] * acc;
    }
    return res;
}
