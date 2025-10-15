#include "utils.hpp"
#include "utils_arrayshapes.hpp"

using ValueAlloca = AllocaInst;
using ValueAllocaVec = std::vector<ValueAlloca *>;

using ValueLoad = LoadInst;
using ValueLoadVec = std::vector<ValueLoad *>;

using SignalGEP = GetElementPtrInst;
using SignalGEPVec = std::vector<SignalGEP *>;

using ConstraintInstance = CallInst;
using ConstraintVec = std::vector<ConstraintInstance *>;

using Component = CallInst;
using ComponentVec = std::vector<Component *>;

const std::string fn_template_prefix = "fn_template_init_";
const std::string fn_build_prefix = "fn_template_build_";
const std::string fn_constraint_prefix = "fn_intrinsic_utils_constraint";
const std::string fn_powi_prefix = "fn_intrinsic_utils_powi";
const std::string fn_switch_prefix = "fn_intrinsic_utils_switch";
const std::string const_inline_array = "const_inline_array";

enum class ValueTypeEnum {
    InputSignal,
    IntermediateSignal,
    OutputSignal,
    ComponentInput,
    ComponentOutput,
    Argument,
    Constant,
    Expression,
    Component,
    Variable,
};

static const ValueTypeEnum VariableTypeAll[] = {
    ValueTypeEnum::InputSignal,     ValueTypeEnum::IntermediateSignal,
    ValueTypeEnum::OutputSignal,    ValueTypeEnum::ComponentInput,
    ValueTypeEnum::ComponentOutput,

    ValueTypeEnum::Argument,        ValueTypeEnum::Constant,
    ValueTypeEnum::Expression,      ValueTypeEnum::Component,
    ValueTypeEnum::Variable,
};

ValueTypeEnum abbrToTypeEnum(std::string s);
std::string typeEnumToAbbr(ValueTypeEnum ve);

// Source expression, Destination expression
using Assignment = std::pair<Value *, Instruction *>;
using AssignmentVec = std::vector<Assignment>;

// variable_name, type_enum.
using CircomValue = std::pair<std::string, ValueTypeEnum>;
// Circom value with concrete access
using CircomValueAccess = std::pair<CircomValue, ArrayShape>;

// LLVM value with ValueTypeEnum
using LLVMValue = std::pair<Value *, ValueTypeEnum>;
//  LLVM value with LLVM access
using LLVMValueAccess = std::pair<LLVMValue, ValueVec>;
using LLVMValueAccessVec = std::vector<LLVMValueAccess>;

bool isInstantiation(Module *M);
std::string extractMainComp(Module *M);

bool isTemplateInitFunc(Function *F);
bool isTemplateBuildFunc(Function *F);
bool isPowiFunc(Function *F);
bool isSwitchFunc(Function *F);

bool isEntry(BasicBlock *bb);
bool isExit(BasicBlock *bb);

bool locatesAtEntry(Instruction *inst);
bool locatesAtExit(Instruction *inst);

// Key instruction
bool isComponent(Instruction *inst);
bool isConstraint(Instruction *inst);
bool isInlineArray(Instruction *inst);
bool isAssignment(Instruction *inst);

// Operator
bool isInitial(Instruction *inst);
bool isRead(Instruction *inst);
bool isWrite(Instruction *inst);

// Circom Value
bool isCircomFormatValue(Value *v);
bool isInputSignal(Instruction *inst);
bool isInterSignal(Instruction *inst);
bool isOutputSignal(Instruction *inst);
bool isComponentInput(Instruction *inst);
bool isComponentOutput(Instruction *inst);
bool isVariable(Instruction *inst);
bool isArgument(Value *v);
bool isConstant(Value *v);
bool isComponentVariable(Instruction *inst);

// Signal
bool isSelfSignal(Instruction *inst);
bool isCompSignal(Instruction *inst);
bool isSignal(Instruction *inst);
bool isReadSignal(Instruction *inst);
bool isWriteSignal(Instruction *inst);

std::pair<std::string, CircomValue> extractValue(Value *v);
std::string extractValueName(Value *v);
std::string extractComponentName(Function *F);
NamePair extractCompSignal(Value *v);
std::string mergeNamePair(NamePair p);
NamePair splitNamePairMerged(std::string s);
std::string normalizeComponentName(std::string s);
Assignment resolveAssignment(Instruction *inst);
LLVMValueAccess resolveSignalOrValue(Value *v);
Value *resolveVariable(Instruction *inst);
class Collector;

using CollectorMap = std::unordered_map<std::string, Collector *>;

class Collector {
   private:
   public:
    Function *F;
    bool is_instantiation;
    std::string template_name;
    std::string comp_name;
    ConstraintVec constraints;
    InstructionVec assignments;

    AssignmentVec assignment_pairs;

    ValueAllocaVec input_signals;
    ValueAllocaVec inter_signals;
    ValueAllocaVec output_signals;
    ValueAllocaVec variables;
    ValueAllocaVec component_variables;

    SignalGEPVec component_inputs;
    SignalGEPVec component_outputs;

    ComponentVec components;

    NameSet argument_names;
    NameSet input_signal_names;
    NameSet inter_signal_names;
    NameSet output_signal_names;
    NameSet variable_names;
    // n2b[0] -> Num2bits@n=128
    NameMap comp_var2comp_names;
    NameSetMap component_input_signal_names;
    NameSetMap component_output_signal_names;

    NameSet flat_component_input_signal_names;
    NameSet flat_component_output_signal_names;

    ArrayShapeMap array_shapes;

    Collector(Function *F);
    
    std::string genNameWithIdx(std::string var_name, uint64_t index);
    NameVec genAllNames(std::string name, ArrayShapePair ap);
    std::string getName();
    ArrayShape getArrayShape(std::string s);
    NameSet getCompInputSignals(std::string comp_name);
    NameSet getCompOutputSignals(std::string comp_name);
    void addCompSignals(CollectorMap global_c);
    json::Object format();
};

std::vector<Collector *> sortCollectors(FunctionVec funcs);
