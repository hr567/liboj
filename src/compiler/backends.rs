use std::collections::HashMap;

use bincode;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const BACKENDS_BINCODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/languages"));

lazy_static! {
    static ref LANGUAGES: HashMap<String, CompilerConfig> =
        bincode::deserialize(BACKENDS_BINCODE).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct CompilerConfig {
    pub suffix: String,
    pub command: String,
    pub args: Vec<String>,
    pub timeout: u64,
}

pub fn get_config(language: &str) -> Option<&'static CompilerConfig> {
    LANGUAGES.get(language)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gcc")]
    mod c_gcc;

    #[cfg(feature = "gxx")]
    mod cpp_gxx;
}
