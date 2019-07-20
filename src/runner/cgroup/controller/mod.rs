mod cpu;
mod cpuacct;
mod memory;

use std::io;

use super::*;

pub use self::{cpu::*, cpuacct::*, memory::*};

/// Cgroup controller trait.
///
/// The controller should not outlive the cgroup context.
pub trait Controller<'a> {
    /// The name of the controller.
    const NAME: &'static str;

    /// Generate a controller from the given context.
    ///
    /// The lifetime make sure that the controller will
    /// not outlive the context.
    fn from_ctx(context: &'a Context) -> Self
    where
        Self: 'a;

    /// Has the controller been initialized.
    ///
    /// A controller may be uninitialized when it is created.
    /// In that case, the user should call `initialize` by hand.
    fn is_initialized(&self) -> bool;

    /// Initialize the controller.
    ///
    /// If the initializing process has failed, it panic.
    ///
    /// The controller will be destroyed by the context
    /// when the context is dropped.
    fn initialize(&self) -> io::Result<()>;
}
