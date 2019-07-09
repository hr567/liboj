//! A high performance framework for building online judge system.
/// A configurable output checker.
pub mod checker;
/// Interface for different compilers.
pub mod compiler;
/// Program runner with resource limit and system calls filter.
pub mod runner;
/// Structures definitions.
pub mod structures;

pub use checker::Checker;
pub use compiler::Compiler;
pub use structures::*;
