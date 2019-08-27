use crate::compiler::Compiler;
use crate::structures::Source;

use std::io;

use tempfile;

#[test]
fn test_cpp_gxx_compile() -> io::Result<()> {
    let source = Source {
        language: String::from("cpp.g++"),
        code: String::from("#include<iostream>\nint main() { return 0; }"),
    };
    let compiler = Compiler::new(&source.language).unwrap();
    let executable_file = tempfile::NamedTempFile::new()?.into_temp_path();
    let compiler_output = compiler.compile(&source, &executable_file)?;
    assert!(compiler_output.status.success());
    Ok(())
}

#[test]
fn test_cpp_gxx_compile_failed() -> io::Result<()> {
    let source = Source {
        language: String::from("cpp.g++"),
        code: String::from("#include<iostream>\nint main() { return 0 }"),
    };
    let compiler = Compiler::new(&source.language).unwrap();
    let executable_file = tempfile::NamedTempFile::new()?.into_temp_path();
    let compiler_output = compiler.compile(&source, &executable_file)?;
    assert!(!compiler_output.status.success());
    Ok(())
}
