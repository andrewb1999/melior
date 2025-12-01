//! Build script for mlir-sys
//!
//! This script handles finding MLIR/LLVM installations and generating Rust bindings.
//!
//! # Configuration
//!
//! The build can be configured via environment variables:
//!
//! ## Using MLIR_SYS_*_PREFIX (recommended for installed LLVM)
//! - `MLIR_SYS_210_PREFIX`: Path to LLVM/MLIR installation or build directory
//!
//! ## Using separate directories (for development builds)
//! - `MLIR_BUILD_DIR` or `LLVM_BUILD_DIR`: Path to LLVM build directory
//! - `MLIR_SRC_DIR`: Path to MLIR source directory (for includes)
//! - `LLVM_SRC_DIR`: Path to LLVM source directory (for includes)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LLVM_MAJOR_VERSION: usize = 21;

fn main() {
    let prefix_var = format!("MLIR_SYS_{}0_PREFIX", LLVM_MAJOR_VERSION);
    println!("cargo:rerun-if-env-changed={}", prefix_var);
    println!("cargo:rerun-if-env-changed=MLIR_BUILD_DIR");
    println!("cargo:rerun-if-env-changed=LLVM_BUILD_DIR");
    println!("cargo:rerun-if-env-changed=MLIR_SRC_DIR");
    println!("cargo:rerun-if-env-changed=LLVM_SRC_DIR");
    println!("cargo:rerun-if-changed=wrapper.h");

    let config = find_mlir_config();

    setup_linking(&config);
    generate_bindings(&config);
}

struct MlirConfig {
    /// Directory containing MLIR/LLVM libraries
    lib_dir: PathBuf,
    /// Include directories for bindgen
    include_dirs: Vec<PathBuf>,
}

fn find_mlir_config() -> MlirConfig {
    // Try MLIR_SYS_*_PREFIX first
    let prefix_var = format!("MLIR_SYS_{}0_PREFIX", LLVM_MAJOR_VERSION);
    if let Ok(prefix) = env::var(&prefix_var) {
        return config_from_prefix(&prefix);
    }

    // Try llvm-config in PATH
    if let Some(config) = try_llvm_config(None) {
        return config;
    }

    // Try separate build/source directories (for development)
    if let (Ok(build_dir), Ok(mlir_src), Ok(llvm_src)) = (
        env::var("MLIR_BUILD_DIR").or_else(|_| env::var("LLVM_BUILD_DIR")),
        env::var("MLIR_SRC_DIR"),
        env::var("LLVM_SRC_DIR"),
    ) {
        return config_from_build_dirs(&build_dir, &mlir_src, &llvm_src);
    }

    panic!(
        "Could not find MLIR/LLVM installation.\n\
         \n\
         Please set one of the following:\n\
         \n\
         1. MLIR_SYS_{major}0_PREFIX - Path to LLVM build or installation directory\n\
            Example: export MLIR_SYS_{major}0_PREFIX=/path/to/llvm-project/build\n\
         \n\
         2. For development builds, set all three:\n\
            - MLIR_BUILD_DIR or LLVM_BUILD_DIR - Path to LLVM build directory\n\
            - MLIR_SRC_DIR - Path to MLIR source directory\n\
            - LLVM_SRC_DIR - Path to LLVM source directory\n\
         \n\
         3. Or ensure llvm-config is in your PATH\n",
        major = LLVM_MAJOR_VERSION
    );
}

fn config_from_prefix(prefix: &str) -> MlirConfig {
    let prefix = Path::new(prefix);

    // Try llvm-config from the prefix first
    let llvm_config = prefix.join("bin").join("llvm-config");
    if llvm_config.exists() {
        if let Some(config) = try_llvm_config(Some(&llvm_config)) {
            return config;
        }
    }

    // Check if this is a build directory or installed prefix
    let lib_dir = prefix.join("lib");
    let mut include_dirs = Vec::new();

    // Check for build directory layout (has tools/mlir/include)
    let mlir_build_include = prefix.join("tools").join("mlir").join("include");
    if mlir_build_include.exists() {
        include_dirs.push(mlir_build_include);
    }

    let build_include = prefix.join("include");
    if build_include.exists() {
        include_dirs.push(build_include);
    }

    // For build directories, try to find source directories
    // by looking for common patterns
    if let Some(llvm_src) = find_llvm_source_from_build(prefix) {
        let llvm_include = llvm_src.join("include");
        if llvm_include.exists() {
            include_dirs.push(llvm_include);
        }

        // MLIR source is usually a sibling of llvm
        if let Some(parent) = llvm_src.parent() {
            let mlir_include = parent.join("mlir").join("include");
            if mlir_include.exists() {
                include_dirs.push(mlir_include);
            }
        }
    }

    // Fall back to just the prefix include if nothing else worked
    if include_dirs.is_empty() {
        include_dirs.push(prefix.join("include"));
    }

    MlirConfig { lib_dir, include_dirs }
}

fn find_llvm_source_from_build(build_dir: &Path) -> Option<PathBuf> {
    // Try to read CMakeCache.txt to find source directory
    let cache_path = build_dir.join("CMakeCache.txt");
    if let Ok(content) = fs::read_to_string(&cache_path) {
        for line in content.lines() {
            if line.starts_with("LLVM_SOURCE_DIR:") || line.starts_with("CMAKE_HOME_DIRECTORY:") {
                if let Some(path) = line.split('=').nth(1) {
                    let path = Path::new(path.trim());
                    if path.exists() {
                        return Some(path.to_path_buf());
                    }
                }
            }
        }
    }

    // Common pattern: build dir is inside or sibling to source
    if let Some(parent) = build_dir.parent() {
        let llvm_src = parent.join("llvm");
        if llvm_src.join("include").exists() {
            return Some(llvm_src);
        }
    }

    None
}

fn try_llvm_config(llvm_config_path: Option<&Path>) -> Option<MlirConfig> {
    let llvm_config = llvm_config_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("llvm-config"));

    let output = Command::new(&llvm_config)
        .args(["--version"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout);
    let major: usize = version.split('.').next()?.parse().ok()?;

    if major != LLVM_MAJOR_VERSION {
        eprintln!(
            "cargo:warning=Found LLVM version {}, expected {}",
            major, LLVM_MAJOR_VERSION
        );
    }

    let lib_dir = run_llvm_config(&llvm_config, &["--libdir"])?;
    let include_dir = run_llvm_config(&llvm_config, &["--includedir"])?;

    let mut include_dirs = vec![PathBuf::from(include_dir.trim())];

    // Check if this looks like a build directory
    let lib_path = Path::new(lib_dir.trim());
    if let Some(build_dir) = lib_path.parent() {
        let mlir_build_include = build_dir.join("tools").join("mlir").join("include");
        if mlir_build_include.exists() {
            include_dirs.insert(0, mlir_build_include);
        }
        let build_include = build_dir.join("include");
        if build_include.exists() && !include_dirs.contains(&build_include) {
            include_dirs.insert(0, build_include);
        }

        // Try to find source includes
        if let Some(llvm_src) = find_llvm_source_from_build(build_dir) {
            if let Some(parent) = llvm_src.parent() {
                let mlir_include = parent.join("mlir").join("include");
                if mlir_include.exists() {
                    include_dirs.push(mlir_include);
                }
            }
        }
    }

    Some(MlirConfig {
        lib_dir: PathBuf::from(lib_dir.trim()),
        include_dirs,
    })
}

fn run_llvm_config(llvm_config: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new(llvm_config).args(args).output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn config_from_build_dirs(build_dir: &str, mlir_src: &str, llvm_src: &str) -> MlirConfig {
    let build_path = Path::new(build_dir);

    MlirConfig {
        lib_dir: build_path.join("lib"),
        include_dirs: vec![
            // Generated headers (highest priority)
            build_path.join("tools").join("mlir").join("include"),
            build_path.join("include"),
            // Source headers
            Path::new(mlir_src).join("include"),
            Path::new(llvm_src).join("include"),
        ],
    }
}

fn setup_linking(config: &MlirConfig) {
    println!(
        "cargo:rustc-link-search=native={}",
        config.lib_dir.display()
    );

    // Discover and link libraries
    let (mlir_libs, llvm_libs) = discover_libraries(&config.lib_dir);

    for lib in &mlir_libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    for lib in &llvm_libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    link_system_libraries();
}

fn discover_libraries(lib_dir: &Path) -> (Vec<String>, Vec<String>) {
    let mut mlir_libs = Vec::new();
    let mut llvm_libs = Vec::new();

    if let Ok(entries) = fs::read_dir(lib_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("libMLIR") && filename.ends_with(".a") {
                    let lib_name = &filename[3..filename.len() - 2];
                    mlir_libs.push(lib_name.to_string());
                } else if filename.starts_with("libLLVM") && filename.ends_with(".a") {
                    let lib_name = &filename[3..filename.len() - 2];
                    llvm_libs.push(lib_name.to_string());
                }
            }
        }
    }

    mlir_libs.sort();
    llvm_libs.sort();

    (mlir_libs, llvm_libs)
}

fn link_system_libraries() {
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=rt");

        // Optional libraries - try to link if available
        for lib in &["tinfo", "ncurses", "ffi", "xml2"] {
            if pkg_config_exists(lib) {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=curses");
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=uuid");
    }
}

fn pkg_config_exists(name: &str) -> bool {
    Command::new("pkg-config")
        .args(["--exists", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn generate_bindings(config: &MlirConfig) {
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allow all MLIR C API symbols
        .allowlist_function("mlir.*")
        .allowlist_type("Mlir.*")
        .allowlist_var("MLIR.*")
        .allowlist_type("MlirLlvmThreadPool")
        .allowlist_function("mlirLlvm.*")
        // Block inline functions (reimplemented in Rust)
        .blocklist_function("mlirStringRefCreate")
        .blocklist_function("mlirLogicalResultIsSuccess")
        .blocklist_function("mlirLogicalResultIsFailure")
        .blocklist_function("mlirLogicalResultSuccess")
        .blocklist_function("mlirLogicalResultFailure")
        .blocklist_function("mlirContextIsNull")
        .blocklist_function("mlirDialectIsNull")
        .blocklist_function("mlirDialectRegistryIsNull")
        .blocklist_function("mlirLocationIsNull")
        .blocklist_function("mlirModuleIsNull")
        .blocklist_function("mlirOperationIsNull")
        .blocklist_function("mlirRegionIsNull")
        .blocklist_function("mlirBlockIsNull")
        .blocklist_function("mlirValueIsNull")
        .blocklist_function("mlirTypeIsNull")
        .blocklist_function("mlirAttributeIsNull")
        .blocklist_function("mlirSymbolTableIsNull")
        .blocklist_function("mlirTypeIDIsNull")
        .blocklist_function("mlirAffineMapIsNull")
        // Block MlirAffineMap - bindgen gets confused by forward declaration
        .blocklist_type("MlirAffineMap")
        // Generate proper Rust types
        .derive_debug(true)
        .derive_copy(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .size_t_is_usize(true)
        .layout_tests(false)
        .use_core()
        .ctypes_prefix("::std::os::raw");

    // Add include directories
    for include_dir in &config.include_dirs {
        builder = builder.clang_arg(format!("-I{}", include_dir.display()));
    }

    let bindings = builder.generate().expect("Failed to generate MLIR bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
