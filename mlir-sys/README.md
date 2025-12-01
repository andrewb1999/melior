# mlir-sys

Low-level Rust bindings for the MLIR C API, generated using [bindgen](https://github.com/rust-lang/rust-bindgen).

This crate is part of the [melior](https://github.com/mlir-rs/melior) project.

## Prerequisites

1. **Built MLIR/LLVM 21**: You need LLVM 21 built with MLIR and the C API enabled.

2. **Clang**: bindgen requires clang for parsing C headers.

## Configuration

The build script can find LLVM/MLIR via several methods:

### Option 1: MLIR_SYS_210_PREFIX (recommended)

Point to your LLVM build or installation directory:

```bash
export MLIR_SYS_210_PREFIX=/path/to/llvm-project/build
cargo build
```

### Option 2: Separate directories

For development builds where sources and build are separate:

```bash
export MLIR_BUILD_DIR=/path/to/llvm-project/build
export MLIR_SRC_DIR=/path/to/llvm-project/mlir
export LLVM_SRC_DIR=/path/to/llvm-project/llvm
cargo build
```

### Option 3: llvm-config in PATH

If `llvm-config` is in your PATH and returns version 21, it will be used automatically:

```bash
# Just ensure llvm-config is available
cargo build
```

## Usage

```rust
use mlir_sys::*;

fn main() {
    unsafe {
        // Create a context
        let ctx = mlirContextCreate();
        
        // Create an unknown location
        let loc = mlirLocationUnknownGet(ctx);
        
        // Create an empty module
        let module = mlirModuleCreateEmpty(loc);
        
        // Parse a type
        let type_str = MlirStringRef::from_str("i32");
        let i32_type = mlirTypeParseGet(ctx, type_str);
        
        // Parse an attribute
        let attr_str = MlirStringRef::from_str("42 : i32");
        let attr = mlirAttributeParseGet(ctx, attr_str);
        
        // Clean up
        mlirModuleDestroy(module);
        mlirContextDestroy(ctx);
    }
}
```

## Safety

All functions in this crate are `unsafe` as they directly call C functions.
For a safe, idiomatic Rust API, use the [melior](https://crates.io/crates/melior) crate.

## Building LLVM/MLIR

Here's how to build LLVM 21 with MLIR support:

```bash
git clone https://github.com/llvm/llvm-project.git
cd llvm-project
git checkout release/21.x  # or main for development

mkdir build && cd build
cmake -G Ninja ../llvm \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_ENABLE_PROJECTS="mlir" \
    -DLLVM_TARGETS_TO_BUILD="host" \
    -DMLIR_ENABLE_BINDINGS_PYTHON=OFF

ninja
```

## Troubleshooting

### Missing libraries

If you get linker errors about missing MLIR libraries:
1. Make sure MLIR was built with the C API enabled
2. Check that `MLIR_SYS_210_PREFIX` points to the correct build directory
3. Verify all required libraries are present in `$MLIR_SYS_210_PREFIX/lib/`

### Header parsing errors

If bindgen fails to parse headers:
1. Make sure clang is installed
2. Check that include paths are correct
3. Try setting `LIBCLANG_PATH` to your clang installation

### Missing system libraries

On Linux, you may need to install development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libncurses5-dev libtinfo-dev libffi-dev libxml2-dev zlib1g-dev

# Fedora/RHEL
sudo dnf install ncurses-devel libffi-devel libxml2-devel zlib-devel
```

## License

Licensed under the Apache License v2.0 with LLVM Exceptions.
