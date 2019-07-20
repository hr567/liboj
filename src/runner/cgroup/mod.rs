//! Reduced high-level APIs for cgroup.
mod controller;
mod hierarchy;

use std::fs::remove_dir;
use std::io;
use std::path::{Path, PathBuf};

use nix::unistd::Pid;
use uuid::Uuid;

pub use controller::*;
pub use hierarchy::*;

/// Provide the root path of a cgroup.
pub trait CgroupRoot {
    fn root() -> &'static Path;
}

/// Cgroup context.
#[derive(Debug, Clone)]
pub struct Context {
    name: String,
}

impl Context {
    /// Create a new cgroup context with a random UUID as its name.
    pub fn new() -> Context {
        Context::default()
    }

    /// Create a new cgroup context with a given name.
    pub fn with_name(name: &str) -> Context {
        Context {
            name: name.to_owned(),
        }
    }

    /// Get the cpu controller.
    ///
    /// The controller must be initialized before read or write to it.
    pub fn cpu_controller(&self) -> CpuController<PathBuf> {
        CpuController::from_ctx(&self)
    }

    /// Get the cpuacct controller.
    ///
    /// The controller must be initialized before read or write to it.
    pub fn cpuacct_controller(&self) -> CpuAcctController<PathBuf> {
        CpuAcctController::from_ctx(&self)
    }

    /// Get the cpuacct controller.
    ///
    /// The controller must be initialized before read or write to it.
    pub fn memory_controller(&self) -> MemoryController<PathBuf> {
        MemoryController::from_ctx(&self)
    }

    /// Add a process to the context.
    pub fn add_process(&self, pid: Pid) -> io::Result<()> {
        for hierarchy in self.hierarchies() {
            hierarchy.procs().write(&pid)?;
        }
        Ok(())
    }

    /// Add a task(thread) to the context.
    pub fn add_task(&self, pid: Pid) -> io::Result<()> {
        for hierarchy in self.hierarchies() {
            hierarchy.tasks().write(&pid)?;
        }
        Ok(())
    }
}

impl Context {
    /// All hierarchies that this cgroup context contains.
    fn hierarchies<'a>(&'a self) -> impl Iterator<Item = Box<dyn 'a + Hierarchy>> {
        let mut res: Vec<Box<dyn Hierarchy>> = Vec::new();
        if self.cpu_controller().is_initialized() {
            res.push(Box::new(self.cpu_controller()));
        }
        if self.cpuacct_controller().is_initialized() {
            res.push(Box::new(self.cpuacct_controller()));
        }
        if self.memory_controller().is_initialized() {
            res.push(Box::new(self.memory_controller()));
        }
        res.into_iter()
    }
}

impl CgroupRoot for Context {
    fn root() -> &'static Path {
        Path::new("/sys/fs/cgroup/")
    }
}

impl Default for Context {
    fn default() -> Context {
        Context::with_name(&Uuid::new_v4().to_string())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        for hierarchy in self.hierarchies() {
            let _ = remove_dir(hierarchy.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;
    use std::time::Duration;

    #[test]
    fn test_cgroup_path() {
        let ctx = Context::new();

        let cpu_controller = ctx.cpu_controller();
        cpu_controller.initialize().unwrap();
        let cpu_path = cpu_controller.as_ref();
        assert!(cpu_path.exists());
        assert_eq!(
            cpu_path,
            PathBuf::from_iter(&["/sys/fs/cgroup/cpu/", ctx.name.as_str()])
        );

        let cpuacct_controller = ctx.cpuacct_controller();
        cpuacct_controller.initialize().unwrap();
        let cpuact_path = cpuacct_controller.as_ref();
        assert!(cpuact_path.exists());
        assert_eq!(
            cpuact_path,
            PathBuf::from_iter(&["/sys/fs/cgroup/cpuacct/", ctx.name.as_str()])
        );

        let memory_controller = ctx.memory_controller();
        memory_controller.initialize().unwrap();
        let memory_path = memory_controller.as_ref();
        assert!(memory_path.exists());
        assert_eq!(
            memory_path,
            PathBuf::from_iter(&["/sys/fs/cgroup/memory/", ctx.name.as_str()])
        );
    }

    #[test]
    fn test_cpu_controller() -> io::Result<()> {
        let ctx = Context::new();

        let cpu_controller = ctx.cpu_controller();
        cpu_controller.initialize()?;

        cpu_controller.period().write(&Duration::from_millis(200))?;
        cpu_controller.quota().write(&Duration::from_millis(80))?;
        assert_eq!(cpu_controller.period().read()?, Duration::from_millis(200));
        assert_eq!(cpu_controller.quota().read()?, Duration::from_millis(80));

        cpu_controller.period().write(&Duration::from_millis(150))?;
        cpu_controller.quota().write(&Duration::from_millis(50))?;
        assert_eq!(cpu_controller.period().read()?, Duration::from_millis(150));
        assert_eq!(cpu_controller.quota().read()?, Duration::from_millis(50));

        Ok(())
    }

    #[test]
    fn test_cpu_acct_controller() -> io::Result<()> {
        let ctx = Context::new();
        let cpuacct_controller = ctx.cpuacct_controller();
        cpuacct_controller.initialize()?;
        let cpu_usage = cpuacct_controller.usage()?;
        assert_eq!(cpu_usage, Duration::from_secs(0));
        Ok(())
    }

    #[test]
    fn test_memory_controller() -> io::Result<()> {
        let ctx = Context::new();

        let memory_controller = ctx.memory_controller();
        memory_controller.initialize()?;

        memory_controller.limit_in_bytes().write(&(128 * 1024))?;
        assert_eq!(memory_controller.limit_in_bytes().read()?, 128 * 1024);

        memory_controller.limit_in_bytes().write(&(128 * 1024))?;
        assert_eq!(memory_controller.limit_in_bytes().read()?, 128 * 1024);

        assert_eq!(memory_controller.usage_in_bytes()?, 0);
        assert_eq!(memory_controller.max_usage_in_bytes()?, 0);
        assert_eq!(memory_controller.failcnt()?, 0);

        Ok(())
    }
}
