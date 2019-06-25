use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CGROUP_ROOT: &str = "/sys/fs/cgroup/";

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Controller {
    Cpu,
    Memory,
}

impl Controller {
    #[allow(unreachable_patterns)]
    pub fn cgroup_path(self) -> PathBuf {
        Path::new(CGROUP_ROOT).join(match self {
            Controller::Cpu => "cpu",
            Controller::Memory => "memory",
            _ => unimplemented!("Unsupported controller"),
        })
    }
}

pub trait Cpu {
    fn cpu_usage(&self) -> io::Result<Duration>;
    fn set_cpu_period(&self, period: Duration) -> io::Result<()>;
    fn set_cpu_quota(&self, quota: Duration) -> io::Result<()>;
}

pub trait Memory {
    fn memory_usage(&self) -> io::Result<usize>;
    fn set_memory_limit(&self, limit: usize) -> io::Result<()>;
    fn set_swappiness(&self, flag: bool) -> io::Result<()>;
    fn memory_failcnt(&self) -> io::Result<usize>;
}
