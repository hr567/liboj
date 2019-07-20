use std::fmt::Debug;
use std::fs::{read_to_string, write};
use std::io;
use std::path::Path;
use std::str::FromStr;

use nix::unistd::Pid;

use super::Controller;

/// Hierarchy in the cgroup.
pub trait Hierarchy<'a> {
    /// The path of this hierarchy.
    fn path(&self) -> &Path;

    /// `cgroup.procs` file in this hierarchy.
    ///
    /// Can read from and write to this it.
    fn procs(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>>;

    /// `cgroup.tasks` file in this hierarchy.
    ///
    /// Can read from and write to this it.
    fn tasks(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>>;
}

impl<'a, T: Controller<'a> + AsRef<Path>> Hierarchy<'a> for T {
    fn path(&self) -> &Path {
        self.as_ref()
    }

    fn procs(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>> {
        Box::new(PidFile::from(self.path().join("cgroup.procs")))
    }

    fn tasks(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>> {
        Box::new(PidFile::from(self.path().join("tasks")))
    }
}

/// A file which can be written and read.
///
/// It is very common in the cgroup hierarchy.
pub trait AttrFile<'a, T, U> {
    /// Write a attribute to the file.
    ///
    /// There is a default implementation for types
    /// which has `ToString` trait.
    fn write(&self, attr: &T) -> io::Result<()>;

    /// Write a attribute to the file.
    ///
    /// There is a default implementation for types
    /// which has `FromStr` trait.
    fn read(&self) -> io::Result<U>;
}

impl<'a, T, U, P> AttrFile<'a, T, T> for P
where
    T: FromStr<Err = U> + ToString,
    U: Debug,
    P: AsRef<Path>,
{
    fn write(&self, attr: &T) -> io::Result<()> {
        write(&self, &attr.to_string())?;
        Ok(())
    }

    fn read(&self) -> io::Result<T> {
        let attr = read_to_string(&self)?.trim().parse().unwrap();
        Ok(attr)
    }
}

struct PidFile<T: AsRef<Path>> {
    inner: T,
}

impl<T: AsRef<Path>> From<T> for PidFile<T> {
    fn from(inner: T) -> PidFile<T> {
        PidFile { inner }
    }
}

impl<'a, T: AsRef<Path>> AttrFile<'a, Pid, Vec<Pid>> for PidFile<T> {
    fn write(&self, pid: &Pid) -> io::Result<()> {
        write(&self.inner, pid.to_string())?;
        Ok(())
    }

    fn read(&self) -> io::Result<Vec<Pid>> {
        let tasks = read_to_string(&self.inner)?
            .split_whitespace()
            .map(|pid| -> Pid { Pid::from_raw(pid.trim().parse().unwrap()) })
            .collect();
        Ok(tasks)
    }
}
