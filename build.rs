use std::collections::HashMap;
use std::env;
use std::fs::{read_dir, read_to_string, File};
use std::path::PathBuf;

use bincode;
use bindgen;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;

lazy_static! {
    static ref OUT_DIR: PathBuf = {
        let out_dir = env::var("OUT_DIR").unwrap();
        PathBuf::from(out_dir)
    };
    static ref ROOT_DIR: PathBuf = {
        let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(root_dir)
    };
}

fn main() {
    generate_compiler_backends();
    generate_seccomp_binding();
}

fn generate_seccomp_binding() {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp_wrapper.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate seccomp bindings")
        .write_to_file(OUT_DIR.join("seccomp_wrapper.rs"))
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct Config {
    suffix: String,
    command: String,
    args: Vec<String>,
    timeout: u64,
}

fn generate_compiler_backends() {
    let backends_dir = ROOT_DIR.join("src/compiler/backends");
    let mut backends: HashMap<String, Config> = HashMap::new();
    read_dir(&backends_dir)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_name().to_str().unwrap().ends_with(".json"))
        .map(|entry| (entry.file_name(), read_to_string(&entry.path()).unwrap()))
        .for_each(|(filename, config)| {
            let language = filename
                .to_str()
                .unwrap()
                .trim_end_matches(".json")
                .to_owned();
            let config: Config = serde_json::from_str(&config)
                .expect(&format!("Configuration file {} is unavailable", &language));
            backends.insert(language, config);
        });
    let backends_file = File::create(OUT_DIR.join("compiler_backends")).unwrap();
    bincode::serialize_into(backends_file, &backends).unwrap();
}
