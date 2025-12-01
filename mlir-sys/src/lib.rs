//! Low-level Rust bindings for the MLIR C API.
//!
//! This crate provides raw FFI bindings to the MLIR C API, generated using bindgen.
//! For a safe, idiomatic Rust API, see the [`melior`](https://crates.io/crates/melior) crate.
//!
//! # Usage
//!
//! ```rust,ignore
//! use mlir_sys::*;
//!
//! unsafe {
//!     let ctx = mlirContextCreate();
//!     // ... use MLIR ...
//!     mlirContextDestroy(ctx);
//! }
//! ```
//!
//! # Configuration
//!
//! The build script can find LLVM/MLIR via several methods:
//!
//! ## 1. MLIR_SYS_*_PREFIX (recommended)
//! Point to your LLVM build or installation directory:
//! ```bash
//! export MLIR_SYS_210_PREFIX=/path/to/llvm-project/build
//! ```
//!
//! ## 2. Separate directories (for development builds)
//! ```bash
//! export MLIR_BUILD_DIR=/path/to/llvm-project/build
//! export MLIR_SRC_DIR=/path/to/llvm-project/mlir  
//! export LLVM_SRC_DIR=/path/to/llvm-project/llvm
//! ```
//!
//! ## 3. llvm-config in PATH
//! If `llvm-config` is available and returns version 21, it will be used automatically.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ============================================================================
// Manually defined types that bindgen has trouble with
// ============================================================================

/// An affine map.
/// 
/// This is manually defined because bindgen gets confused by the forward
/// declaration in AffineExpr.h before the full definition in AffineMap.h.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MlirAffineMap {
    pub ptr: *const ::std::os::raw::c_void,
}

// ============================================================================
// Re-implement inline functions from the C headers
// These are blocked in bindgen because they're defined inline in the headers
// ============================================================================

/// Constructs a string reference from the pointer and length.
#[inline]
pub fn mlirStringRefCreate(str: *const ::std::os::raw::c_char, length: usize) -> MlirStringRef {
    MlirStringRef {
        data: str,
        length,
    }
}

/// Constructs a string reference from a Rust string slice.
#[inline]
pub fn mlirStringRefCreateFromStr(s: &str) -> MlirStringRef {
    MlirStringRef {
        data: s.as_ptr() as *const ::std::os::raw::c_char,
        length: s.len(),
    }
}

/// Checks if the given logical result represents a success.
#[inline]
pub fn mlirLogicalResultIsSuccess(res: MlirLogicalResult) -> bool {
    res.value != 0
}

/// Checks if the given logical result represents a failure.
#[inline]
pub fn mlirLogicalResultIsFailure(res: MlirLogicalResult) -> bool {
    res.value == 0
}

/// Creates a logical result representing a success.
#[inline]
pub fn mlirLogicalResultSuccess() -> MlirLogicalResult {
    MlirLogicalResult { value: 1 }
}

/// Creates a logical result representing a failure.
#[inline]
pub fn mlirLogicalResultFailure() -> MlirLogicalResult {
    MlirLogicalResult { value: 0 }
}

/// Checks whether a context is null.
#[inline]
pub fn mlirContextIsNull(context: MlirContext) -> bool {
    context.ptr.is_null()
}

/// Checks if the dialect is null.
#[inline]
pub fn mlirDialectIsNull(dialect: MlirDialect) -> bool {
    dialect.ptr.is_null()
}

/// Checks if the dialect registry is null.
#[inline]
pub fn mlirDialectRegistryIsNull(registry: MlirDialectRegistry) -> bool {
    registry.ptr.is_null()
}

/// Checks if the location is null.
#[inline]
pub fn mlirLocationIsNull(location: MlirLocation) -> bool {
    location.ptr.is_null()
}

/// Checks whether a module is null.
#[inline]
pub fn mlirModuleIsNull(module: MlirModule) -> bool {
    module.ptr.is_null()
}

/// Checks whether the underlying operation is null.
#[inline]
pub fn mlirOperationIsNull(op: MlirOperation) -> bool {
    op.ptr.is_null()
}

/// Checks whether a region is null.
#[inline]
pub fn mlirRegionIsNull(region: MlirRegion) -> bool {
    region.ptr.is_null()
}

/// Checks whether a block is null.
#[inline]
pub fn mlirBlockIsNull(block: MlirBlock) -> bool {
    block.ptr.is_null()
}

/// Returns whether the value is null.
#[inline]
pub fn mlirValueIsNull(value: MlirValue) -> bool {
    value.ptr.is_null()
}

/// Checks whether a type is null.
#[inline]
pub fn mlirTypeIsNull(type_: MlirType) -> bool {
    type_.ptr.is_null()
}

/// Checks whether an attribute is null.
#[inline]
pub fn mlirAttributeIsNull(attr: MlirAttribute) -> bool {
    attr.ptr.is_null()
}

/// Returns true if the symbol table is null.
#[inline]
pub fn mlirSymbolTableIsNull(symbolTable: MlirSymbolTable) -> bool {
    symbolTable.ptr.is_null()
}

/// Checks whether a type id is null.
#[inline]
pub fn mlirTypeIDIsNull(typeID: MlirTypeID) -> bool {
    typeID.ptr.is_null()
}

/// Checks whether an affine map is null.
#[inline]
pub fn mlirAffineMapIsNull(affineMap: MlirAffineMap) -> bool {
    affineMap.ptr.is_null()
}

// ============================================================================
// Helper types and functions for easier Rust usage
// ============================================================================

impl MlirStringRef {
    /// Create a new MlirStringRef from a Rust string slice.
    ///
    /// # Safety
    /// The returned MlirStringRef borrows from the input string,
    /// so the string must outlive the MlirStringRef.
    #[inline]
    pub fn from_str(s: &str) -> Self {
        mlirStringRefCreateFromStr(s)
    }

    /// Create a new MlirStringRef from a C string pointer and length.
    #[inline]
    pub fn new(data: *const ::std::os::raw::c_char, length: usize) -> Self {
        MlirStringRef { data, length }
    }

    /// Convert to a Rust string slice.
    ///
    /// # Safety
    /// The data pointer must be valid and point to valid UTF-8 data.
    #[inline]
    pub unsafe fn as_str(&self) -> &str {
        let slice = std::slice::from_raw_parts(self.data as *const u8, self.length);
        std::str::from_utf8_unchecked(slice)
    }

    /// Convert to a Rust string slice, checking for valid UTF-8.
    ///
    /// # Safety
    /// The data pointer must be valid.
    #[inline]
    pub unsafe fn as_str_checked(&self) -> Result<&str, std::str::Utf8Error> {
        let slice = std::slice::from_raw_parts(self.data as *const u8, self.length);
        std::str::from_utf8(slice)
    }

    /// Check if the string reference is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get the length of the string reference.
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }
}

impl MlirLogicalResult {
    /// Check if this result represents success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.value != 0
    }

    /// Check if this result represents failure.
    #[inline]
    pub fn is_failure(&self) -> bool {
        self.value == 0
    }

    /// Create a success result.
    #[inline]
    pub fn success() -> Self {
        MlirLogicalResult { value: 1 }
    }

    /// Create a failure result.
    #[inline]
    pub fn failure() -> Self {
        MlirLogicalResult { value: 0 }
    }
}

impl From<bool> for MlirLogicalResult {
    fn from(b: bool) -> Self {
        if b {
            MlirLogicalResult::success()
        } else {
            MlirLogicalResult::failure()
        }
    }
}

impl From<MlirLogicalResult> for bool {
    fn from(res: MlirLogicalResult) -> Self {
        res.is_success()
    }
}

impl From<MlirLogicalResult> for Result<(), ()> {
    fn from(res: MlirLogicalResult) -> Self {
        if res.is_success() {
            Ok(())
        } else {
            Err(())
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_ref() {
        let s = "hello";
        let sr = MlirStringRef::from_str(s);
        assert_eq!(sr.len(), 5);
        assert!(!sr.is_empty());
        unsafe {
            assert_eq!(sr.as_str(), "hello");
        }
    }

    #[test]
    fn test_logical_result() {
        let success = MlirLogicalResult::success();
        assert!(success.is_success());
        assert!(!success.is_failure());

        let failure = MlirLogicalResult::failure();
        assert!(!failure.is_success());
        assert!(failure.is_failure());

        let from_bool: MlirLogicalResult = true.into();
        assert!(from_bool.is_success());
    }

    #[test]
    fn test_context_creation() {
        unsafe {
            let ctx = mlirContextCreate();
            assert!(!mlirContextIsNull(ctx));
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_location_creation() {
        unsafe {
            let ctx = mlirContextCreate();
            let loc = mlirLocationUnknownGet(ctx);
            assert!(!mlirLocationIsNull(loc));
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_module_creation() {
        unsafe {
            let ctx = mlirContextCreate();
            let loc = mlirLocationUnknownGet(ctx);
            let module = mlirModuleCreateEmpty(loc);
            assert!(!mlirModuleIsNull(module));
            mlirModuleDestroy(module);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_type_parsing() {
        unsafe {
            let ctx = mlirContextCreate();
            let type_str = MlirStringRef::from_str("i32");
            let ty = mlirTypeParseGet(ctx, type_str);
            assert!(!mlirTypeIsNull(ty));
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_attribute_parsing() {
        unsafe {
            let ctx = mlirContextCreate();
            let attr_str = MlirStringRef::from_str("42 : i32");
            let attr = mlirAttributeParseGet(ctx, attr_str);
            assert!(!mlirAttributeIsNull(attr));
            mlirContextDestroy(ctx);
        }
    }
}
