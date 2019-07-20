use std::fs::{create_dir, read_to_string};
use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::time::Duration;

use super::*;

pub struct CpuAcctController<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a ()>,
}

impl<'a, T: 'a + AsRef<Path>> CpuAcctController<'a, T> {
    pub fn usage(&self) -> io::Result<Duration> {
        let file = self.inner.as_ref().join("cpuacct.usage");
        let usage = read_to_string(&file)?.trim().parse().unwrap();
        Ok(Duration::from_nanos(usage))
    }

    pub fn usage_all(&self) -> io::Result<Vec<(usize, Duration, Duration)>> {
        let file = self.inner.as_ref().join("cpuacct.usage_all");
        let usages = read_to_string(&file)?
            .lines()
            .skip(1) // The first line is "cpu user system"
            .map(|line| line.split_whitespace().collect())
            .map(|line: Vec<&str>| (line[0], line[1], line[2]))
            .map(|(index, usage_user, usage_sys)| {
                (
                    index.parse().unwrap(),
                    usage_user.parse().unwrap(),
                    usage_sys.parse().unwrap(),
                )
            })
            .map(|(index, usage_user, usage_sys)| {
                (
                    index,
                    Duration::from_nanos(usage_user),
                    Duration::from_nanos(usage_sys),
                )
            })
            .collect();
        Ok(usages)
    }

    pub fn usage_percpu(&self) -> io::Result<Vec<Duration>> {
        let file = self.inner.as_ref().join("cpuacct.usage_percpu");
        let usages = read_to_string(&file)?
            .split_whitespace()
            .map(|usage| usage.parse().unwrap())
            .map(Duration::from_nanos)
            .collect();
        Ok(usages)
    }

    pub fn usage_percpu_sys(&self) -> io::Result<Vec<Duration>> {
        let file = self.inner.as_ref().join("cpuacct.usage_percpu_sys");
        let usages = read_to_string(&file)?
            .split_whitespace()
            .map(|usage| usage.parse().unwrap())
            .map(Duration::from_nanos)
            .collect();
        Ok(usages)
    }

    pub fn usage_percpu_user(&self) -> io::Result<Vec<Duration>> {
        let file = self.inner.as_ref().join("cpuacct.usage_percpu_user");
        let usages = read_to_string(&file)?
            .split_whitespace()
            .map(|usage| usage.parse().unwrap())
            .map(Duration::from_nanos)
            .collect();
        Ok(usages)
    }

    pub fn usage_sys(&self) -> io::Result<Duration> {
        let file = self.inner.as_ref().join("cpuacct.usage_sys");
        let usage = read_to_string(&file)?.trim().parse().unwrap();
        Ok(Duration::from_nanos(usage))
    }

    pub fn usage_user(&self) -> io::Result<Duration> {
        let file = self.inner.as_ref().join("cpuacct.usage_user");
        let usage = read_to_string(&file)?.trim().parse().unwrap();
        Ok(Duration::from_nanos(usage))
    }
}

impl<'a, T: 'a + AsRef<Path> + From<PathBuf>> Controller<'a> for CpuAcctController<'a, T> {
    const NAME: &'static str = "cpuacct";

    fn from_ctx(context: &Context) -> CpuAcctController<T> {
        let inner = Context::root().join(Self::NAME).join(&context.name);
        CpuAcctController {
            inner: inner.into(),
            _mark: PhantomData,
        }
    }

    fn initialize(&self) -> io::Result<()> {
        if !self.is_initialized() {
            create_dir(&self.inner)?;
        }
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.inner.as_ref().exists()
    }
}

impl<'a, T: 'a + AsRef<Path>> AsRef<Path> for CpuAcctController<'a, T> {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
