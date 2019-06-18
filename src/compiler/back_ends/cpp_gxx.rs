use std::io;
use std::path;
use std::process;
use std::thread;
use std::time;

use super::*;

pub struct CppGxx;

impl Compiler for CppGxx {
    const SOURCE_SUFFIX: &'static str = "cpp";

    fn compile(source_file: &path::Path, executable_file: &path::Path) -> io::Result<bool> {
        let source_file = rename_with_new_extension(&source_file, CppGxx::SOURCE_SUFFIX)?;

        let mut child = process::Command::new("g++")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .arg(source_file.as_os_str())
            .arg("-o")
            .arg(executable_file.as_os_str())
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c++11")
            .spawn()?;

        let start_compiling_time = time::Instant::now();
        let compile_success = loop {
            if start_compiling_time.elapsed().as_millis() >= 5000 {
                child.kill()?;
                break false;
            }
            if let Some(status) = child.try_wait()? {
                break status.success();
            }
            thread::sleep(time::Duration::from_millis(100));
        };
        Ok(compile_success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile;

    #[test]
    fn test_cpp_gxx_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("cpp_compiler_test_pass.cpp");
        let executable_file = work_dir.path().join("cpp_compiler_test_pass.exe");
        fs::write(&source_file, "#include<iostream>\nint main() { return 0; }").unwrap();
        let compile_success = CppGxx::compile(&source_file, &executable_file);
        assert!(compile_success.unwrap());
    }

    #[test]
    fn test_cpp_gxx_compile_failed() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("cpp_compiler_test_fail.cpp");
        let executable_file = work_dir.path().join("cpp_compiler_test_fail.exe");
        fs::write(&source_file, "#include<iostream>\nint main() { return 0 }").unwrap();
        let compile_success = CppGxx::compile(&source_file, &executable_file);
        assert!(!compile_success.unwrap());
    }
}
