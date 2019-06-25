//! Interface for different compilers.
use crate::structures::Source;

mod backends;

use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json;

/// A simple compiler interface for compile a single file to a executable file.
pub struct Compiler {
    /// Suffix which source file should be used.
    suffix: String,
    /// Command of the compiler in system.
    command: String,
    /// Arguments of the compiler.
    args: Vec<String>,
    /// Timeout of the compiler.
    timeout: Duration,
}

impl Compiler {
    /// Generate a new compiler for `language`.
    ///
    /// Choose a configuration from backends to build
    /// a suitable compiler for the language and return it.
    /// Or return `None` if the language is not support.
    ///
    /// Return an 'Err` if there is a configuration for the `language`
    /// but the configuration is unavailable or there is an io error.
    pub fn new(language: &str) -> Option<Result<Compiler, Box<dyn Error>>> {
        let config: Config = match Compiler::load_config(language) {
            Some(Ok(config)) => config,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        };

        Some(Ok(config.into()))
    }

    /// Return configuration for `language`.
    /// Or return `None` if the language is not support,
    /// `Some(Err)` if the configuration is unavailable
    fn load_config(language: &str) -> Option<Result<Config, Box<dyn Error>>> {
        let json_file = Path::new("src")
            .join("compiler")
            .join("backends")
            .join(format!("{}.json", language));
        if !json_file.exists() {
            return None;
        }
        let json_file = match File::open(&json_file) {
            Ok(file) => file,
            Err(e) => return Some(Err(Box::new(e))),
        };

        let config: Config = match serde_json::from_reader(json_file) {
            Ok(config) => config,
            Err(e) => return Some(Err(Box::new(e))),
        };

        Some(Ok(config))
    }

    /// Compile `source` to `executable_file`.
    ///
    /// A temporary file whose file name ends with `suffix`
    /// will be created to save the source.
    /// The temporary file will be removed after the compiling process.
    ///
    /// Return the result of the compiler process,
    /// or return `Err` if the command run incorrectly.
    pub fn compile(&self, source: &Source, executable_file: &Path) -> io::Result<bool> {
        let mut source_file = tempfile::Builder::new()
            .prefix("source_")
            .suffix(&format!(".{}", &self.suffix))
            .tempfile()?;
        source_file.write_all(source.code.as_bytes())?;
        let source_file = source_file.into_temp_path();

        let mut child = Command::new(&self.command)
            .args(self.args.iter().map(|arg| match arg.as_str() {
                "{source_file}" => source_file.as_os_str().to_owned(),
                "{executable_file}" => executable_file.as_os_str().to_owned(),
                _ => OsString::from(arg),
            }))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let start_time = Instant::now();
        let result = loop {
            match child.try_wait()? {
                Some(status) => break status,
                None => {
                    if start_time.elapsed() > self.timeout {
                        child.kill()?;
                    } else {
                        sleep(Duration::from_millis(100));
                    }
                }
            }
        };

        Ok(result.success())
    }
}

/// A helper structures for compiler configuration.
#[derive(Serialize, Deserialize)]
struct Config {
    suffix: String,
    command: String,
    args: Vec<String>,
    timeout: u64,
}

impl Into<Compiler> for Config {
    fn into(self) -> Compiler {
        let Config {
            suffix,
            command,
            args,
            timeout,
        } = self;

        Compiler {
            suffix,
            command,
            args,
            timeout: Duration::from_secs(timeout),
        }
    }
}
