mod backends;

use std::ffi::OsString;
use std::io::{self, prelude::*};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use crate::executor::ChildExt as _;
use crate::structures::Source;

/// A simple compiler interface for compile a single file to a executable file.
pub struct Compiler<'a> {
    /// Suffix which source file should be used.
    suffix: &'a str,
    /// Command of the compiler in system.
    command: &'a str,
    /// Arguments of the compiler.
    args: &'a Vec<String>,
    /// Timeout of the compiler.
    timeout: Duration,
}

impl<'a> From<&'a backends::CompilerConfig> for Compiler<'a> {
    fn from(config: &'a backends::CompilerConfig) -> Compiler<'a> {
        Compiler {
            suffix: &config.suffix,
            command: &config.command,
            args: &config.args,
            timeout: Duration::from_secs(config.timeout),
        }
    }
}

impl<'a> Compiler<'a> {
    /// Generate a new compiler for `language`.
    ///
    /// Choose a configuration from backends to build
    /// a suitable compiler for the language and return it.
    /// Or return `None` if the language is not support.
    ///
    /// Return an `Err` if there is a configuration for the `language`
    /// but the configuration is unavailable or there is an io error.
    pub fn new(language: &str) -> Option<Compiler> {
        Some(Compiler::from(backends::get_config(language)?))
    }

    /// Compile `source` to `executable_file`.
    ///
    /// A temporary file whose file name ends with `suffix`
    /// will be created to save the source.
    /// The temporary file will be removed after the compiling process.
    ///
    /// Return the result of the compiler process,
    /// or return `Err` if the command run incorrectly.
    pub fn compile(&self, source: &Source, executable_file: &Path) -> io::Result<Output> {
        let source_file = {
            let mut res = tempfile::Builder::new()
                .prefix("source_")
                .suffix(&format!(".{}", &self.suffix))
                .tempfile()?;
            res.write_all(source.code.as_bytes())?;
            res.into_temp_path()
        };

        let output = Command::new(&self.command)
            .args(self.args.iter().map(|arg| match arg.as_str() {
                "{source_file}" => source_file.as_os_str().to_owned(),
                "{executable_file}" => executable_file.as_os_str().to_owned(),
                _ => OsString::from(arg),
            }))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?
            .timeout_with_output(self.timeout)?;

        Ok(output)
    }
}
