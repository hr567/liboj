//! Program runner with resource limit and system call filter.
// TODO: Add tests
// TODO: Need improvements
mod cgroup;
mod seccomp;

use cgroup::*;
use seccomp::*;

// use std::borrow::ToOwned;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use std::os::unix::{
    ffi::OsStrExt as _,
    io::{AsRawFd as _, IntoRawFd as _},
};

use nix;

use crate::structures::Resource;

/// Report of the program
#[derive(Debug)]
pub struct RunnerReport {
    /// set to `true` if the program exit with code zero and
    /// set to `false` if exited with non-zero code or be killed
    pub exit_success: bool,

    /// time the program use in real world
    pub resource_usage: Option<Resource>,
}

pub struct Runner {
    program: PathBuf,
    input_file: PathBuf,
    output_file: PathBuf,
    chroot: Option<PathBuf>,
    cgroup: Option<Cgroup>,
    seccomp: Option<ScmpCtx>,
    resource_limit: Option<Resource>,
}

impl Runner {
    pub fn new(
        program: impl AsRef<Path>,
        input_file: impl AsRef<Path>,
        output_file: impl AsRef<Path>,
    ) -> Runner {
        Runner {
            program: program.as_ref().to_owned(),
            input_file: input_file.as_ref().to_owned(),
            output_file: output_file.as_ref().to_owned(),
            chroot: None,
            cgroup: None,
            seccomp: None,
            resource_limit: None,
        }
    }

    pub fn chroot(mut self, chroot_path: impl AsRef<Path>) -> Runner {
        self.chroot = Some(chroot_path.as_ref().to_owned());
        self
    }

    pub fn seccomp(mut self, seccomp: ScmpCtx) -> Runner {
        self.seccomp = Some(seccomp);
        self
    }

    pub fn cgroup(mut self, cgroup: Cgroup) -> Runner {
        self.cgroup = Some(cgroup);
        self
    }

    pub fn resource_limit(mut self, limit: Resource) -> Runner {
        self.resource_limit = Some(limit);
        self
    }

    pub fn run(&self) -> Result<RunnerReport, Box<dyn Error>> {
        match nix::unistd::fork()? {
            nix::unistd::ForkResult::Parent { child } => self.start_parent(child),
            nix::unistd::ForkResult::Child => {
                self.start_child()?;
                unreachable!("After fork and exec child process")
            }
        }
    }
}

impl Runner {
    fn start_parent(&self, child: nix::unistd::Pid) -> Result<RunnerReport, Box<dyn Error>> {
        let start_time = Instant::now();
        if let Some(cg) = &self.cgroup {
            cg.add_process(child)?;
        }
        if let Some(limit) = &self.resource_limit {
            let time_limit = limit.real_time;
            if time_limit != Duration::from_secs(0) {
                thread::spawn(move || {
                    thread::sleep(time_limit);
                    nix::sys::signal::kill(child, nix::sys::signal::SIGKILL).unwrap_or_default();
                });
            }
        }
        let exit_success = match nix::sys::wait::waitpid(child, None)? {
            nix::sys::wait::WaitStatus::Exited(_, code) => code == 0,
            nix::sys::wait::WaitStatus::Signaled(_, _, _) => false,
            _ => unreachable!("Should not appear other cases"),
        };
        let resource_usage = match &self.cgroup {
            Some(cg) => Some(Resource::new(
                cg.cpu_usage()?,
                start_time.elapsed(),
                cg.memory_usage()?,
            )),
            None => None,
        };
        Ok(RunnerReport {
            exit_success,
            resource_usage,
        })
    }

    fn start_child(&self) -> Result<(), Box<Error>> {
        let input_fd = File::open(&self.input_file)?.into_raw_fd();
        nix::unistd::dup2(input_fd, io::stdin().as_raw_fd())?;
        nix::unistd::close(input_fd)?;

        let output_fd = File::create(&self.output_file)?.into_raw_fd();
        nix::unistd::dup2(output_fd, io::stdout().as_raw_fd())?;
        nix::unistd::close(output_fd)?;

        self.unshare_namespace()?;
        nix::unistd::chdir("/")?;
        if let Some(chroot_dir) = &self.chroot {
            nix::unistd::chroot(chroot_dir.as_path())?;
        }
        let program_command = CString::new(self.program.as_os_str().as_bytes()).unwrap();
        self.init_seccomp(program_command.as_ptr())?;
        nix::unistd::execvpe(&program_command, &[program_command.clone()], &[])?;
        unreachable!("Not reachable after exec")
    }

    fn unshare_namespace(&self) -> nix::Result<()> {
        nix::sched::unshare(
            nix::sched::CloneFlags::empty()
                | nix::sched::CloneFlags::CLONE_FILES
                | nix::sched::CloneFlags::CLONE_FS
                | nix::sched::CloneFlags::CLONE_NEWCGROUP
                | nix::sched::CloneFlags::CLONE_NEWIPC
                | nix::sched::CloneFlags::CLONE_NEWNET
                | nix::sched::CloneFlags::CLONE_NEWNS
                | nix::sched::CloneFlags::CLONE_NEWPID
                | nix::sched::CloneFlags::CLONE_NEWUSER
                | nix::sched::CloneFlags::CLONE_NEWUTS
                | nix::sched::CloneFlags::CLONE_SYSVSEM,
        )
    }

    fn init_seccomp(&self, program_ptr: *const i8) -> Result<(), Box<dyn Error>> {
        if let Some(scmp) = &self.seccomp {
            scmp.whitelist(seccomp::syscall_resolve_name("execve").unwrap(), {
                let mut pattern = seccomp::Pattern::new();
                pattern.add_arg(seccomp::CmpOp::EQ, program_ptr as i64);
                pattern
            })?;
            scmp.load()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
