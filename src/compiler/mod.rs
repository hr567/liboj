//! Interface for different compilers.
mod back_ends;
use back_ends::get_compile_fn;

use std::io;
use std::path;

/// Compile a single source file to the executable file.
/// The language of the source file must be given first.
///
/// Return `Ok(res)` if any one of the compilers support the language.
/// `res` is the result of the compiler.
/// If the language is not support by any compiler,
/// then a `Err()` will be returned.
pub fn compile(
    language: &str,
    source_file: &path::Path,
    executable_file: &path::Path,
) -> Result<io::Result<bool>, &'static str> {
    let compile_fn = get_compile_fn(&language)?;
    Ok(compile_fn(&source_file, &executable_file))
}
