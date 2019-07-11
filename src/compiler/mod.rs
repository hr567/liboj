//! Interface for different compilers.
#[cfg(test)]
mod test_backends {
    //! Supported compilers.
    #[cfg(feature = "gcc")]
    mod c_gcc;

    #[cfg(feature = "gxx")]
    mod cpp_gxx;
}

mod backends {
    use std::collections::HashMap;

    use lazy_static::lazy_static;
    use serde_json;

    const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/compiler_backends.json"));

    lazy_static! {
        pub static ref COMPILERS: HashMap<String, String> =
            { serde_json::from_str(CONFIG_JSON).unwrap() };
    }
}

use std::ffi::OsString;
use std::io::{self, prelude::*};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::structures::Source;
use backends::COMPILERS as compilers;
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
    pub fn new(language: &str) -> Option<Compiler> {
        let config: Config = Compiler::load_config(language)?;
        Some(config.into())
    }

    /// Return configuration for `language`.
    /// Or return `None` if the language is not support,
    /// `Some(Err)` if the configuration is unavailable
    fn load_config(language: &str) -> Option<Config> {
        let config_json = compilers.get(language)?;
        Some(serde_json::from_str(&config_json).unwrap())
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
