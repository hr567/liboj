use std::collections::HashMap;
use std::env;
use std::fs::{read_dir, read_to_string, write};
use std::io;
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

fn main() -> io::Result<()> {
    watch_changes()?;
    generate_compiler_backends()?;
    generate_seccomp_binding()?;
    Ok(())
}

fn watch_changes() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", "Cargo.toml");
    read_dir(&ROOT_DIR.join("src/compiler/backends"))?.for_each(|backend| {
        println!(
            "cargo:rerun-if-changed={}",
            backend.unwrap().path().to_str().unwrap()
        );
    });
    println!("cargo:rerun-if-changed=/usr/include/seccomp.h");
    Ok(())
}

fn generate_seccomp_binding() -> io::Result<()> {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp_wrapper.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate seccomp bindings")
        .write_to_file(OUT_DIR.join("seccomp_wrapper.rs"))?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    suffix: String,
    command: String,
    args: Vec<String>,
    timeout: u64,
}

fn generate_compiler_backends() -> io::Result<()> {
    let backends: HashMap<String, Config> = read_dir(&ROOT_DIR.join("src/compiler/backends"))?
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_name().to_str().unwrap().ends_with(".json"))
        .map(|entry| (entry.file_name(), read_to_string(&entry.path()).unwrap()))
        .map(|(filename, config)| {
            let language = filename
                .to_str()
                .unwrap()
                .trim_end_matches(".json")
                .to_owned();
            let config: Config = serde_json::from_str(&config)
                .expect(&format!("Configuration file {} is unavailable", &language));
            (language, config)
        })
        .collect();
    write(
        OUT_DIR.join("compiler_backends"),
        bincode::serialize(&backends).unwrap(),
    )?;
    Ok(())
}
