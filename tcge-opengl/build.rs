extern crate gl_generator;

use gl_generator::{Registry, Fallbacks, StructGenerator, DebugStructGenerator, Api, Profile};
use std::env;
use std::path::Path;
use std::fs::{copy, File};

fn main() {
    println!("--- tcge-client-gl: build.rs ---");
    
    let crt_dir = env::var("CARGO_MANIFEST_DIR").expect("Env-Var 'CARGO_MANIFEST_DIR' is missing.");
    let out_dir = env::var("OUT_DIR").expect("Env-Var 'OUT_DIR' is missing.");
    let src_dir = &Path::new(&crt_dir).join("src");
    
    // Ensure that cargo only re-runs this script if `lib.rs` changes...
    let lib_src = src_dir.clone().join("lib.rs");
    println!("cargo:rerun-if-changed={}", lib_src.as_path().to_str().unwrap_or("[ERROR]"));
    
    // Define which OpenGL version to use...
    let registry = Registry::new(
        Api::Gl,
        (4, 5),
        Profile::Core,
        Fallbacks::All,
        []
    );
    
    // Pick future path to `bindings.rs`...
    let file_gl_path = &Path::new(&out_dir).join("bindings.rs");
    let mut file_gl = File::create(file_gl_path).unwrap();
    
    // Actually generate bindings...
    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        registry.write_bindings(
            DebugStructGenerator,
            &mut file_gl
        ).unwrap();
    } else {
        registry.write_bindings(
            StructGenerator,
            &mut file_gl
        ).unwrap();
    }
    
    eprintln!("Path to OpenGL bindings is: {}", file_gl_path.as_path().to_str().unwrap_or("[ERROR]"));
    
    let bindings_file_src = file_gl_path;
    let bindings_file_dst = src_dir.clone().join("bindings.rs");
    
    eprintln!("Copy from {} to {}",
        bindings_file_src.as_path().to_str().unwrap_or("[ERROR]"),
        bindings_file_dst.as_path().to_str().unwrap_or("[ERROR]")
    );
    
    copy(
        bindings_file_src,
        bindings_file_dst
    ).expect("Could not copy generated bindings.rs into src directory.");
}

