use std::io;
use std::path;
use std::process;
use std::thread;
use std::time;

use super::*;

pub struct CGcc;

impl Compiler for CGcc {
    const SOURCE_SUFFIX: &'static str = "c";

    fn compile(source_file: &path::Path, executable_file: &path::Path) -> io::Result<bool> {
        let source_file = rename_with_new_extension(&source_file, CGcc::SOURCE_SUFFIX)?;

        let mut child = process::Command::new("gcc")
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
            .arg("--std=c99")
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
    fn test_c_gcc_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_pass.c");
        let executable_file = work_dir.path().join("c_compiler_test_pass.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0; }").unwrap();
        let compile_success = CGcc::compile(&source_file, &executable_file);
        assert!(compile_success.unwrap());
    }

    #[test]
    fn test_c_gcc_compile_failed() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_fail.c");
        let executable_file = work_dir.path().join("c_compiler_test_fail.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0 }").unwrap();
        let compile_success = CGcc::compile(&source_file, &executable_file);
        assert!(!compile_success.unwrap());
    }
}
