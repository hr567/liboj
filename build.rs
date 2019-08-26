use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use bincode;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;

#[cfg(any(feature = "seccomp", feature = "cap-ng"))]
use bindgen;

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
    #[cfg(feature = "seccomp")]
    generate_libseccomp_binding()?;
    #[cfg(feature = "cap-ng")]
    generate_libcap_ng_binding()?;
    Ok(())
}

fn watch_changes() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", "Cargo.toml");
    fs::read_dir(&ROOT_DIR.join("src/compiler/backends"))?.for_each(|backend| {
        println!(
            "cargo:rerun-if-changed={}",
            backend.unwrap().path().to_str().unwrap()
        );
    });
    println!("cargo:rerun-if-changed=/usr/include/seccomp.h");
    println!("cargo:rerun-if-changed=/usr/include/cap-ng.h");
    Ok(())
}

fn generate_compiler_backends() -> io::Result<()> {
    #[derive(Serialize, Deserialize)]
    struct CompilerConfig {
        suffix: String,
        command: String,
        args: Vec<String>,
        timeout: u64,
    }

    let backends: HashMap<String, CompilerConfig> =
        fs::read_dir(&ROOT_DIR.join("src/compiler/backends"))?
            .map(|entry| entry.unwrap())
            .filter(|entry| entry.file_name().to_str().unwrap().ends_with(".json"))
            .map(|entry| {
                (
                    entry.file_name(),
                    fs::read_to_string(&entry.path()).unwrap(),
                )
            })
            .map(|(filename, config)| {
                let language = filename
                    .to_str()
                    .unwrap()
                    .trim_end_matches(".json")
                    .to_owned();
                let config = serde_json::from_str(&config)
                    .expect(&format!("Configuration file {} is unavailable", &language));
                (language, config)
            })
            .collect();

    fs::write(
        OUT_DIR.join("compiler_backends"),
        bincode::serialize(&backends).unwrap(),
    )?;

    Ok(())
}

#[cfg(feature = "seccomp")]
fn generate_libseccomp_binding() -> io::Result<()> {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate libseccomp bindings")
        .write_to_file(OUT_DIR.join("libseccomp.rs"))?;
    Ok(())
}

#[cfg(feature = "cap-ng")]
fn generate_libcap_ng_binding() -> io::Result<()> {
    println!("cargo:rustc-link-lib=dylib=cap-ng");
    bindgen::builder()
        .header_contents("cap-ng.h", "#include<cap-ng.h>")
        .generate()
        .expect("Failed to generate libcap-ng bindings")
        .write_to_file(OUT_DIR.join("libcapng.rs"))?;
    Ok(())
}
