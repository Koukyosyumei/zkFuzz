#include <ctime>
#include <iostream>
#include <memory>
#include <optional>
#include <regex>
#include <sstream>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <vector>

#include "llvm/IR/BasicBlock.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Value.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/raw_ostream.h"

using namespace llvm;

using NameSet = std::unordered_set<std::string>;
using NamePair = std::pair<std::string, std::string>;
using NameVec = std::vector<std::string>;
using NameMap = std::unordered_map<std::string, std::string>;
using NameSetMap = std::unordered_map<std::string, NameSet>;
using NameIdx = std::unordered_map<std::string, size_t>;

using ValueVec = std::vector<Value *>;
using InstructionVec = std::vector<Instruction *>;
using BasicBlockVec = std::vector<BasicBlock *>;
using FunctionVec = std::vector<Function *>;

using StringVec = std::vector<std::string>;

const std::string detecting_str = "Detecting: ";
const std::regex number_suffix("\\.?\\d*$");
const std::regex array_index_suffix("\\[\\d+\\]$");

std::string getNameOrAsOperand(Value *v);

StringVec stringSplit(std::string s, std::string splitor, int times);
FunctionVec sortFunctions(Module *M);
std::string removeIdxInLLVMName(std::string s);
std::string removeIdxInSignalName(std::string s);
std::string formatNameSet(NameSet s);
bool endsWith(const std::string &str, const std::string &suffix);

void printLLVMValue(Value *v);
void initTimer();
double checkTimer();
json::Value obj2value(json::Object obj);

class UnionFind {
   private:
    std::vector<size_t> root;
    std::vector<size_t> rank;

   public:
    UnionFind(size_t num);
    size_t find(size_t x);
    void merge(size_t i, size_t j);
};

template <typename NamedValue>
bool compareNamedValue(NamedValue *v1, NamedValue *v2) {
    int i = v1->getName().compare(v2->getName());
    return i < 0;
}

template <typename T>
std::unordered_set<T> makeIntersection(const std::unordered_set<T> &in1,
                                       const std::unordered_set<T> &in2) {
    if (in2.size() < in1.size()) {
        return makeIntersection(in2, in1);
    }

    std::unordered_set<T> out;
    auto e = in2.end();
    for (auto v : in1) {
        if (in2.find(v) != e) {
            out.insert(v);
        }
    }
    return out;
}

template <typename Iterator, typename T>
std::unordered_set<T> makeUnion(const Iterator &in1, const Iterator &in2) {
    std::unordered_set<T> out;
    out.insert(in1.begin(), in1.end());
    out.insert(in2.begin(), in2.end());
    return out;
}

template <typename T>
int getIndex(const std::vector<T> &v, const T &ele) {
    auto it = find(v.begin(), v.end(), ele);
    if (it != v.end()) {
        return it - v.begin();
    } else {
        return -1;
    }
}
