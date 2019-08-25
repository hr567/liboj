//! A high performance framework for building online judge system.

/// A configurable output checker.
pub mod checker;
/// Interface for different compilers.
pub mod compiler;
/// Executor for running a single program with resource limit and system calls filter.
pub mod executor;
/// Structures definitions.
pub mod structures;

pub use checker::Checker;
pub use compiler::Compiler;
pub use structures::*;
