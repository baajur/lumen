lumen_cc_library(
  NAME
    EIRToLLVM
  HDRS
    "AggregateOpConversions.h"
    "BinaryOpConversions.h"
    "BuiltinOpConversions.h"
    "ComparisonOpConversions.h"
    "ConstantOpConversions.h"
    "ControlFlowOpConversions.h"
    "ConversionSupport.h"
    "ConvertEIRToLLVM.h"
    "FuncLikeOpConversions.h"
    "MapOpConversions.h"
    "MathOpConversions.h"
    "MemoryOpConversions.h"
  SRCS
    "AggregateOpConversions.cpp"
    "BinaryOpConversions.cpp"
    "BuiltinOpConversions.cpp"
    "ComparisonOpConversions.cpp"
    "ConstantOpConversions.cpp"
    "ControlFlowOpConversions.cpp"
    "ConversionSupport.cpp"
    "ConvertEIRToLLVM.cpp"
    "FuncLikeOpConversions.cpp"
    "MapOpConversions.cpp"
    "MathOpConversions.cpp"
    "MemoryOpConversions.cpp"
  DEPS
    lumen::compiler::Dialect::EIR::IR::EIREncodingGen
    lumen::compiler::Dialect::EIR::IR
    lumen::compiler::Target
    MLIRLLVMIR
    MLIRIR
    MLIRPass
    MLIRStandardOps
    MLIRTransforms
  ALWAYSLINK
  PUBLIC
)
