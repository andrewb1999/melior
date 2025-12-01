use std::{env, error::Error, path::Path, process::Command, str};

const LLVM_MAJOR_VERSION: usize = 21;

fn main() -> Result<(), Box<dyn Error>> {
    let version_variable = format!("MLIR_SYS_{LLVM_MAJOR_VERSION}0_PREFIX");

    println!("cargo:rerun-if-env-changed={version_variable}");
    println!("cargo:rerun-if-env-changed=MLIR_SRC_DIR");
    
    // Get LLVM include directory from llvm-config
    let llvm_include_dir = llvm_config("--includedir", &version_variable)?;
    println!("cargo:rustc-env=LLVM_INCLUDE_DIRECTORY={}", llvm_include_dir);
    
    // Get MLIR source include directory - either from MLIR_SRC_DIR env var
    // or by inferring from the LLVM include path (sibling directory)
    let mlir_include_dir = if let Ok(mlir_src) = env::var("MLIR_SRC_DIR") {
        format!("{}/include", mlir_src)
    } else {
        // Try to infer MLIR path from LLVM include path
        // LLVM include: /path/to/llvm-project/llvm/include
        // MLIR include: /path/to/llvm-project/mlir/include  
        let llvm_path = Path::new(&llvm_include_dir);
        if let Some(parent) = llvm_path.parent() {
            if let Some(grandparent) = parent.parent() {
                grandparent.join("mlir").join("include").display().to_string()
            } else {
                llvm_include_dir.clone()
            }
        } else {
            llvm_include_dir.clone()
        }
    };
    println!("cargo:rustc-env=MLIR_INCLUDE_DIRECTORY={}", mlir_include_dir);

    Ok(())
}

fn llvm_config(
    argument: &str,
    version_variable: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let prefix = env::var(version_variable)
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
    let call = format!(
        "{} --link-static {}",
        prefix.join("llvm-config").display(),
        argument
    );

    Ok(str::from_utf8(
        &if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &call]).output()?
        } else {
            Command::new("sh").arg("-c").arg(&call).output()?
        }
        .stdout,
    )?
    .trim()
    .to_string())
}
