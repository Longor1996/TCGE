extern crate walkdir;
extern crate git_version;

use std::env;
use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    println!("Setting version...");
    git_version::set_env();

    println!("Copying assets...");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // locate executable path even if the project is in workspace

    let executable_path = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").unwrap());
    
    println!("Executable Path: {}", executable_path.to_str().unwrap_or("ERROR"));
    
    copy(
        &manifest_dir.join("assets"),
        &executable_path.join("assets"),
    );
    
    let target_triple = env::var("TARGET").unwrap_or("".to_string());
    
    if target_triple.contains("windows") {
        let lib_name = "glfw3.dll";
        
        let lib_dir = PathBuf::from(env!("GLFW_LIB_DIR"));
        let lib_path = lib_dir.join(lib_name);
        println!("GLFW Lib Dir: {}", lib_dir.to_str().unwrap());
        println!("GLFW Lib: {}", lib_path.to_str().unwrap());
        
        fs::copy(lib_path.as_path(), executable_path.join(lib_name)).expect("Failed to copy GLFW-lib");
    }
}

fn locate_target_dir_from_output_dir(mut target_dir_search: &Path) -> Option<&Path> {
    loop {
        // if path ends with "target", we assume this is correct dir
        if target_dir_search.ends_with("target") {
            return Some(target_dir_search);
        }

        // otherwise, keep going up in tree until we find "target" dir
        target_dir_search = match target_dir_search.parent() {
            Some(path) => path,
            None => break,
        }
    }

    None
}

fn copy(from: &Path, to: &Path) {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.into();
    for entry in WalkDir::new(from_path.clone()) {
        let entry = entry.unwrap();

        if let Ok(rel_path) = entry.path().strip_prefix(&from_path) {
            let target_path = to_path.join(rel_path);

            if entry.file_type().is_dir() {
                DirBuilder::new()
                    .recursive(true)
                    .create(target_path).expect("failed to create target dir");
            } else {
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    }
}