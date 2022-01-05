// Compiles the `rsmpi` C shim library.
extern crate cc;
// Generates the Rust header for the C API.
extern crate bindgen;
// Finds out information about the MPI library

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;


/// Result of a successfull probe
#[allow(clippy::manual_non_exhaustive)]
#[derive(Clone, Debug)]
pub struct Library {
    /// Names of the native MPI libraries that need to be linked
    pub libs: Vec<String>,
    /// Search path for native MPI libraries
    pub lib_paths: Vec<PathBuf>,
    /// Search path for C header files
    pub include_paths: Vec<PathBuf>,
    /// The version of the MPI library
    pub version: String,
}

impl Default for Library {
    fn default() -> Self {
        Library {
            libs: vec!["mpi".to_string()],
            lib_paths: vec![
                PathBuf::from("/foo/bar")
            ],
            include_paths: vec![
                PathBuf::from("/foo/bar")
            ],
            version: "foo".to_string()
        }
    }
}

fn main() {

    //  Map for parsing Environment Configuration
    let mut args = HashMap::new();
    args.insert("1", true);
    args.insert("0", false);

    let unix_x86_64_ompi = Library {
        libs: vec![
            "mpi".to_string()
        ],
        lib_paths: vec![
            PathBuf::from("/usr/lib/x86_64-linux-gnu/openmpi/lib")
        ],
        include_paths: vec![
            PathBuf::from("/usr/lib/x86_64-linux-gnu/openmpi/include/openmpi"),
            PathBuf::from("/usr/lib/x86_64-linux-gnu/openmpi/include"),
        ],
        version: "unknown".to_string()
    };

    let archer2_x86_64_cray_mpich = Library {
        libs: vec![
            "mpi".to_string()
        ],
        lib_paths: vec![
            PathBuf::from("/opt/cray/pe/mpich/8.1.4/ofi/AOCC/2.2/lib/"),
            PathBuf::from("/opt/cray/pe/mpich/8.1.4/ofi/AOCC/2.2/lib-abi-mpich/"),
            PathBuf::from("/opt/AMD/aocc-compiler-2.2.0/lib/")
        ],
        include_paths: vec![
            PathBuf::from("/opt/cray/pe/mpich/8.1.4/ofi/AOCC/2.2/include/"),
            PathBuf::from("/opt/AMD/aocc-compiler-2.2.0/include/")
        ],
        version: "unknown".to_string()
    };

    let mut builder = cc::Build::new();
    let mut lib = Library::default();

    let cray = match env::var_os("CRAY") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$CRAY is not set")
    };

    let cray = args.get(&*cray).unwrap();

    if *cray == true {
        // Archer2 MPICH compiler wrapper
        builder.compiler("cc");
        lib = archer2_x86_64_cray_mpich;
    } else {
        // Available on most OpenMPI compatible systems
        builder.compiler("mpicc");
        lib = unix_x86_64_ompi;
    }

    builder.file("src/rsmpi.c");

    for inc in &lib.include_paths {
        builder.include(inc);
    }

    let compiler = builder.try_get_compiler();

    // Build the `rsmpi` C shim library
    builder.compile("rsmpi");

    // Let `rustc` know about the library search directories.
    for dir in &lib.lib_paths {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    for lib in &lib.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let mut builder = bindgen::builder();
    // Let `bindgen` know about header search directories.
    for dir in &lib.include_paths {
        builder = builder.clang_arg(format!("-I{}", dir.display()));
    }

    // Get the same system includes as used to build the "rsmpi" lib. This block only really does
    // anything when targeting msvc.
    if let Ok(compiler) = compiler {
        let include_env = compiler.env().iter().find(|(key, _)| key == "INCLUDE");
        if let Some((_, include_paths)) = include_env {
            if let Some(include_paths) = include_paths.to_str() {
                // Add include paths via -I
                builder = builder.clang_args(include_paths.split(';').map(|i| format!("-I{}", i)));
            }
        }
    }

    // Generate Rust bindings for the MPI C API.
    let bindings = builder
        .header("src/rsmpi.h")
        .emit_builtins()
        .generate()
        .unwrap();

    // Write the bindings to disk.
    let out_dir = env::var("OUT_DIR").expect("cargo did not set OUT_DIR");
    let out_file = Path::new(&out_dir).join("functions_and_types.rs");
    bindings.write_to_file(out_file).unwrap();
}
