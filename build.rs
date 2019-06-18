use bindgen;

fn main() {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp_wrapper.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate seccomp bindings")
        .write_to_file("src/runner/seccomp/warpper.rs")
        .expect("Failed to write to seccomp ffi file");
}
