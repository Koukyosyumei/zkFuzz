#include "utils.hpp"

std::clock_t _timer_start;

std::string getNameOrAsOperand(Value *v) {
  if (!v->getName().empty())
    return std::string(v->getName());
 
  std::string BBName;
  raw_string_ostream OS(BBName);
  v->printAsOperand(OS, false);
  return OS.str();
}

StringVec stringSplit(std::string s, std::string splitor, int times = INT_MAX) {
    auto res = StringVec();
    size_t pos = 0;
    while (((pos = s.find(splitor)) != std::string::npos) && (times > 0)) {
        auto ss = s.substr(0, pos);
        res.push_back(ss);
        s.erase(0, pos + splitor.length());
        times -= 1;
    }
    res.push_back(s);
    return res;
}

std::vector<Function *> sortFunctions(Module *M) {
    auto ordered_functions = std::vector<Function *>();
    for (auto &F : M->functions()) {
        ordered_functions.push_back(&F);
    }
    sort(ordered_functions, compareNamedValue<Function>);
    return ordered_functions;
}

std::string removeIdxInLLVMName(std::string s) {
    s = std::regex_replace(s, number_suffix, "");
    return s;
}

std::string removeIdxInSignalName(std::string s) {
    s = std::regex_replace(s, array_index_suffix, "");
    return s;
}

std::string formatNameSet(NameSet s) {
    std::string str = "{";
    for (auto e : s) {
        str += e;
        str += ", ";
    }
    str += "}";
    return str;
}

bool endsWith(const std::string &str, const std::string &suffix) {
    return str.size() >= suffix.size() &&
           0 == str.compare(str.size() - suffix.size(), suffix.size(), suffix);
}

void printLLVMValue(Value *v) {
    if (v == nullptr) {
        std::cerr << "It is an nullptr";
        return;
    }
    std::cerr << "Name: " << getNameOrAsOperand(v) << ",";
    v->print(errs());
    std::cerr << "\n";
}

void initTimer() { _timer_start = std::clock(); }

double checkTimer() {
    double duration = (std::clock() - _timer_start) / (double)CLOCKS_PER_SEC;
    return duration;
}

json::Value obj2value(json::Object obj) {
    return json::Value(json::Object(obj));
}

size_t UnionFind::find(size_t x) {
    return x == this->root[x] ? x : (this->root[x] = this->find(this->root[x]));
}

void UnionFind::merge(size_t i, size_t j) {
    size_t x = this->find(i), y = this->find(j);
    if (rank[x] <= rank[y])
        root[x] = y;
    else
        root[y] = x;
    if (rank[x] == rank[y] && x != y) rank[y]++;
}

UnionFind::UnionFind(size_t num) {
    for (size_t i = 0; i < num; i++) {
        this->root.push_back(i);
        this->rank.push_back(1);
    }
}
