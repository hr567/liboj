use std::env;
use std::path::{Path, PathBuf};

use bindgen;
use lazy_static::lazy_static;

lazy_static! {
    static ref OUT_DIR: PathBuf = {
        let out_dir = env::var("OUT_DIR").unwrap();
        Path::new(&out_dir).to_owned()
    };
}

fn main() {
    generate_seccomp_binding();
}

fn generate_seccomp_binding() {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp_wrapper.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate seccomp bindings")
        .write_to_file(OUT_DIR.join("seccomp_wrapper.rs"))
        .expect("Failed to write to seccomp wrapper file");
}
