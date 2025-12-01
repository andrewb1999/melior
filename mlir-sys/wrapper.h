/*
 * Master wrapper header for MLIR C API bindings
 * 
 * This file includes all the MLIR C API headers that we want to generate
 * Rust bindings for. Add or remove headers as needed.
 */

#ifndef MLIR_SYS_WRAPPER_H
#define MLIR_SYS_WRAPPER_H

/* Core MLIR C API */
#include "mlir-c/Support.h"
#include "mlir-c/IR.h"
#include "mlir-c/AffineExpr.h"
#include "mlir-c/AffineMap.h"
#include "mlir-c/BuiltinAttributes.h"
#include "mlir-c/BuiltinTypes.h"
#include "mlir-c/Diagnostics.h"
#include "mlir-c/IntegerSet.h"
#include "mlir-c/Interfaces.h"
#include "mlir-c/Pass.h"
#include "mlir-c/Transforms.h"
#include "mlir-c/Rewrite.h"
#include "mlir-c/Debug.h"
#include "mlir-c/Conversion.h"
#include "mlir-c/RegisterEverything.h"
#include "mlir-c/ExecutionEngine.h"

/* All dialect headers - included for full melior compatibility */
#include "mlir-c/Dialect/Func.h"
#include "mlir-c/Dialect/Arith.h"
#include "mlir-c/Dialect/SCF.h"
#include "mlir-c/Dialect/Linalg.h"
#include "mlir-c/Dialect/Tensor.h"
#include "mlir-c/Dialect/MemRef.h"
#include "mlir-c/Dialect/Vector.h"
#include "mlir-c/Dialect/GPU.h"
#include "mlir-c/Dialect/LLVM.h"
#include "mlir-c/Dialect/Math.h"
#include "mlir-c/Dialect/Index.h"
#include "mlir-c/Dialect/Async.h"
#include "mlir-c/Dialect/ControlFlow.h"
#include "mlir-c/Dialect/PDL.h"
#include "mlir-c/Dialect/Quant.h"
#include "mlir-c/Dialect/Shape.h"
#include "mlir-c/Dialect/SparseTensor.h"
#include "mlir-c/Dialect/Transform.h"
#include "mlir-c/Dialect/SPIRV.h"
#include "mlir-c/Dialect/NVVM.h"
#include "mlir-c/Dialect/ROCDL.h"
#include "mlir-c/Dialect/OpenMP.h"
#include "mlir-c/Dialect/NVGPU.h"
#include "mlir-c/Dialect/AMDGPU.h"
#include "mlir-c/Dialect/MLProgram.h"
#include "mlir-c/Dialect/EmitC.h"
#include "mlir-c/Dialect/IRDL.h"
#include "mlir-c/Dialect/SMT.h"

#endif /* MLIR_SYS_WRAPPER_H */
