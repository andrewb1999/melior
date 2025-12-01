//! Affine Dialect Matrix Multiplication Example
//!
//! This example demonstrates how to build an affine dialect matrix multiplication
//! kernel using melior. It creates a function that multiplies two matrices A[M,K]
//! and B[K,N] to produce C[M,N] using nested affine.for loops.
//!
//! The generated MLIR code looks like:
//! ```mlir
//! func.func @matmul(%A: memref<4x8xf32>, %B: memref<8x16xf32>, %C: memref<4x16xf32>) {
//!   affine.for %i = 0 to 4 {
//!     affine.for %j = 0 to 16 {
//!       affine.for %k = 0 to 8 {
//!         %a = affine.load %A[%i, %k] : memref<4x8xf32>
//!         %b = affine.load %B[%k, %j] : memref<8x16xf32>
//!         %c = affine.load %C[%i, %j] : memref<4x16xf32>
//!         %prod = arith.mulf %a, %b : f32
//!         %sum = arith.addf %c, %prod : f32
//!         affine.store %sum, %C[%i, %j] : memref<4x16xf32>
//!       }
//!     }
//!   }
//!   return
//! }
//! ```

use melior::{
    dialect::{arith, func, DialectRegistry},
    ir::{
        attribute::{IntegerAttribute, StringAttribute, TypeAttribute},
        operation::{OperationBuilder, OperationLike},
        r#type::{FunctionType, MemRefType},
        Block, BlockLike, Identifier, Location, Module, Region, RegionLike, Type, Value,
    },
    utility::register_all_dialects,
    Context,
};

/// Load all necessary dialects
fn load_dialects(context: &Context) {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();
}

/// Creates an affine.for operation
///
/// affine.for %i = lower_bound to upper_bound {
///   body
/// }
fn affine_for<'c>(
    context: &'c Context,
    lower_bound: i64,
    upper_bound: i64,
    step: i64,
    body_region: Region<'c>,
    location: Location<'c>,
) -> melior::ir::Operation<'c> {
    // Create empty lower bound map: () -> (lower_bound)
    // This is the constant lower bound affine map
    let lower_bound_map = melior::ir::Attribute::parse(context, &format!("affine_map<() -> ({})>", lower_bound))
        .expect("valid lower bound affine map");
    let upper_bound_map = melior::ir::Attribute::parse(context, &format!("affine_map<() -> ({})>", upper_bound))
        .expect("valid upper bound affine map");
    
    OperationBuilder::new("affine.for", location)
        .add_attributes(&[
            (Identifier::new(context, "lowerBoundMap"), lower_bound_map),
            (Identifier::new(context, "upperBoundMap"), upper_bound_map),
            (
                Identifier::new(context, "step"),
                IntegerAttribute::new(Type::index(context), step).into(),
            ),
        ])
        .add_regions([body_region])
        .build()
        .expect("valid affine.for operation")
}

/// Creates an affine.load operation
///
/// %val = affine.load %memref[%i, %j] : memref<MxNxf32>
fn affine_load<'c>(
    context: &'c Context,
    memref: Value<'c, '_>,
    indices: &[Value<'c, '_>],
    result_type: Type<'c>,
    location: Location<'c>,
) -> melior::ir::Operation<'c> {
    // Create identity affine map for the indices
    let num_dims = indices.len();
    let dims: Vec<String> = (0..num_dims).map(|i| format!("d{}", i)).collect();
    let map_str = format!("affine_map<({}) -> ({})>", dims.join(", "), dims.join(", "));
    let affine_map = melior::ir::Attribute::parse(context, &map_str)
        .expect("valid identity affine map");
    
    let mut operands = vec![memref];
    operands.extend_from_slice(indices);
    
    OperationBuilder::new("affine.load", location)
        .add_attributes(&[(Identifier::new(context, "map"), affine_map)])
        .add_operands(&operands)
        .add_results(&[result_type])
        .build()
        .expect("valid affine.load operation")
}

/// Creates an affine.store operation
///
/// affine.store %val, %memref[%i, %j] : memref<MxNxf32>
fn affine_store<'c>(
    context: &'c Context,
    value: Value<'c, '_>,
    memref: Value<'c, '_>,
    indices: &[Value<'c, '_>],
    location: Location<'c>,
) -> melior::ir::Operation<'c> {
    // Create identity affine map for the indices
    let num_dims = indices.len();
    let dims: Vec<String> = (0..num_dims).map(|i| format!("d{}", i)).collect();
    let map_str = format!("affine_map<({}) -> ({})>", dims.join(", "), dims.join(", "));
    let affine_map = melior::ir::Attribute::parse(context, &map_str)
        .expect("valid identity affine map");
    
    let mut operands = vec![value, memref];
    operands.extend_from_slice(indices);
    
    OperationBuilder::new("affine.store", location)
        .add_attributes(&[(Identifier::new(context, "map"), affine_map)])
        .add_operands(&operands)
        .build()
        .expect("valid affine.store operation")
}

/// Creates an affine.yield operation (for terminating affine.for body)
fn affine_yield<'c>(location: Location<'c>) -> melior::ir::Operation<'c> {
    OperationBuilder::new("affine.yield", location)
        .build()
        .expect("valid affine.yield operation")
}

/// Build a matrix multiplication function using affine dialect
fn build_matmul_module(m: i64, k: i64, n: i64) -> String {
    let context = Context::new();
    load_dialects(&context);

    let location = Location::unknown(&context);
    let module = Module::new(location);

    let f32_type = Type::float32(&context);
    let index_type = Type::index(&context);
    
    // Matrix types: A[M,K], B[K,N], C[M,N]
    let a_type = MemRefType::new(f32_type, &[m, k], None, None);
    let b_type = MemRefType::new(f32_type, &[k, n], None, None);
    let c_type = MemRefType::new(f32_type, &[m, n], None, None);

    // Build the function
    let function = {
        // Function block takes 3 memref arguments
        let function_block = Block::new(&[
            (a_type.into(), location),
            (b_type.into(), location),
            (c_type.into(), location),
        ]);

        let arg_a: Value = function_block.argument(0).unwrap().into();
        let arg_b: Value = function_block.argument(1).unwrap().into();
        let arg_c: Value = function_block.argument(2).unwrap().into();

        // Build nested loops
        let i_loop = build_nested_loops(&context, arg_a, arg_b, arg_c, m, k, n, index_type, f32_type, location);
        function_block.append_operation(i_loop);

        function_block.append_operation(func::r#return(&[], location));

        let function_region = Region::new();
        function_region.append_block(function_block);

        func::func(
            &context,
            StringAttribute::new(&context, "matmul"),
            TypeAttribute::new(
                FunctionType::new(
                    &context,
                    &[a_type.into(), b_type.into(), c_type.into()],
                    &[],
                )
                .into(),
            ),
            function_region,
            &[],
            location,
        )
    };

    module.body().append_operation(function);

    // Verify the module
    if !module.as_operation().verify() {
        eprintln!("Module verification failed!");
    }

    module.as_operation().to_string()
}

/// Build the nested affine.for loops for matmul
fn build_nested_loops<'c>(
    context: &'c Context,
    arg_a: Value<'c, '_>,
    arg_b: Value<'c, '_>,
    arg_c: Value<'c, '_>,
    m: i64,
    k: i64,
    n: i64,
    index_type: Type<'c>,
    f32_type: Type<'c>,
    location: Location<'c>,
) -> melior::ir::Operation<'c> {
    // Build from innermost to outermost
    // innermost: for idx_k = 0 to k
    // middle: for idx_j = 0 to n  
    // outermost: for idx_i = 0 to m

    // i loop block
    let i_block = Block::new(&[(index_type, location)]);
    let idx_i: Value = i_block.argument(0).unwrap().into();

    // j loop region (inside i loop)
    let j_region = {
        let j_block = Block::new(&[(index_type, location)]);
        let idx_j: Value = j_block.argument(0).unwrap().into();

        // k loop region (inside j loop)
        let k_region = {
            let k_block = Block::new(&[(index_type, location)]);
            let idx_k: Value = k_block.argument(0).unwrap().into();

            // Load A[i, k]
            let load_a = k_block.append_operation(affine_load(
                context,
                arg_a,
                &[idx_i, idx_k],
                f32_type,
                location,
            ));
            let val_a: Value = load_a.result(0).unwrap().into();

            // Load B[k, j]
            let load_b = k_block.append_operation(affine_load(
                context,
                arg_b,
                &[idx_k, idx_j],
                f32_type,
                location,
            ));
            let val_b: Value = load_b.result(0).unwrap().into();

            // Load C[i, j]
            let load_c = k_block.append_operation(affine_load(
                context,
                arg_c,
                &[idx_i, idx_j],
                f32_type,
                location,
            ));
            let val_c: Value = load_c.result(0).unwrap().into();

            // prod = a * b
            let mul_op = k_block.append_operation(arith::mulf(val_a, val_b, location));
            let prod: Value = mul_op.result(0).unwrap().into();

            // sum = c + prod
            let add_op = k_block.append_operation(arith::addf(val_c, prod, location));
            let sum: Value = add_op.result(0).unwrap().into();

            // Store sum to C[i, j]
            k_block.append_operation(affine_store(
                context,
                sum,
                arg_c,
                &[idx_i, idx_j],
                location,
            ));

            // Terminate with affine.yield
            k_block.append_operation(affine_yield(location));

            let region = Region::new();
            region.append_block(k_block);
            region
        };

        // Create k loop
        let k_loop = affine_for(context, 0, k, 1, k_region, location);
        j_block.append_operation(k_loop);
        j_block.append_operation(affine_yield(location));

        let region = Region::new();
        region.append_block(j_block);
        region
    };

    // Create j loop
    let j_loop = affine_for(context, 0, n, 1, j_region, location);
    i_block.append_operation(j_loop);
    i_block.append_operation(affine_yield(location));

    let i_region = Region::new();
    i_region.append_block(i_block);

    // Create i loop (outermost)
    affine_for(context, 0, m, 1, i_region, location)
}

fn main() {
    // Matrix dimensions: A[4,8] * B[8,16] = C[4,16]
    let m = 4;
    let k = 8;
    let n = 16;

    println!("Generating affine matmul for A[{}x{}] * B[{}x{}] = C[{}x{}]", m, k, k, n, m, n);
    println!();
    
    let mlir_output = build_matmul_module(m, k, n);
    println!("{}", mlir_output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul_4x8x16() {
        let output = build_matmul_module(4, 8, 16);
        assert!(output.contains("func.func @matmul"));
        assert!(output.contains("affine.for"));
        assert!(output.contains("affine.load"));
        assert!(output.contains("affine.store"));
        assert!(output.contains("arith.mulf"));
        assert!(output.contains("arith.addf"));
    }

    #[test]
    fn test_small_matmul() {
        let output = build_matmul_module(2, 3, 4);
        assert!(output.contains("memref<2x3xf32>"));
        assert!(output.contains("memref<3x4xf32>"));
        assert!(output.contains("memref<2x4xf32>"));
    }
}
