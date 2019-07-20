use std::fs::{create_dir, read_to_string, write};
use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::time::Duration;

use super::*;

pub struct CpuController<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a ()>,
}

impl<'a, T: 'a + AsRef<Path>> CpuController<'a, T> {
    pub fn period(&'a self) -> Box<dyn AttrFile<'a, Duration, Duration> + 'a> {
        Box::new(CpuTimeFile {
            inner: self.inner.as_ref().join("cpu.cfs_period_us"),
            _mark: PhantomData,
        })
    }

    pub fn quota(&'a self) -> Box<dyn AttrFile<'a, Duration, Duration> + 'a> {
        Box::new(CpuTimeFile {
            inner: self.inner.as_ref().join("cpu.cfs_quota_us"),
            _mark: PhantomData,
        })
    }
}

impl<'a, T: 'a + AsRef<Path> + From<PathBuf>> Controller<'a> for CpuController<'a, T> {
    const NAME: &'static str = "cpu";

    fn from_ctx(context: &Context) -> CpuController<T> {
        let inner = Context::root().join(Self::NAME).join(&context.name);
        CpuController {
            inner: inner.into(),
            _mark: PhantomData,
        }
    }

    fn is_initialized(&self) -> bool {
        self.inner.as_ref().exists()
    }

    fn initialize(&self) -> io::Result<()> {
        if !self.is_initialized() {
            create_dir(&self.inner)?;
        }
        Ok(())
    }
}

impl<'a, T: 'a + AsRef<Path>> AsRef<Path> for CpuController<'a, T> {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

struct CpuTimeFile<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a T>,
}

impl<'a, T: 'a + AsRef<Path>> AttrFile<'a, Duration, Duration> for CpuTimeFile<'a, T> {
    fn read(&self) -> io::Result<Duration> {
        let attr: u64 = read_to_string(&self.inner)?.trim().parse().unwrap();
        Ok(Duration::from_micros(attr))
    }

    fn write(&self, attr: &Duration) -> io::Result<()> {
        write(&self.inner, attr.as_micros().to_string())?;
        Ok(())
    }
}
