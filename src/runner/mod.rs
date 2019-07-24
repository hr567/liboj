// TODO: Add tests
//! Program runner with resource limit and system call filter.
//!
//! # Usage:
//!
//! ```no_run
//! # use std::time::*;
//! # use std::io;
//! # use liboj::*;
//! let runner = Runner::new(
//!     "/bin/wc",
//!     "/dev/null",
//!     "/dev/null",
//!     Resource::new(
//!         Duration::from_secs(2),  // Real time limit
//!         Duration::from_secs(1),  // CPU time limit
//!         16 * 1024 * 1024,        // Memory limit
//!     ),
//! );
//! let report = runner.run().unwrap();
//! assert!(report.exit_success);
//! ```

pub mod cgroup;
pub mod seccomp;

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

use crate::structures::Resource;
use cgroup::Controller;
use nix;

/// Report of the program.
#[derive(Debug)]
pub struct RunnerReport {
    /// set to `true` if the program exit with code zero and
    /// set to `false` if exited with non-zero code or be killed
    pub exit_success: bool,

    /// time the program use in real world
    pub resource_usage: Resource,
}

/// Program runner with resource limit, cgroup, chroot and seccomp support.
pub struct Runner {
    program: PathBuf,
    input_file: PathBuf,
    output_file: PathBuf,
    resource_limit: Resource,
    cgroup: cgroup::Context,
    chroot: Option<PathBuf>,
    seccomp: Option<seccomp::Context>,
}

impl Runner {
    /// Create a new runner with resource limit.
    pub fn new(
        program: impl AsRef<Path>,
        input_file: impl AsRef<Path>,
        output_file: impl AsRef<Path>,
        resource_limit: Resource,
    ) -> Runner {
        Runner {
            program: program.as_ref().to_owned(),
            input_file: input_file.as_ref().to_owned(),
            output_file: output_file.as_ref().to_owned(),
            resource_limit,
            cgroup: cgroup::Context::new(),
            chroot: None,
            seccomp: None,
        }
    }

    /// Chroot to the given path.
    pub fn chroot(mut self, chroot_path: impl AsRef<Path>) -> Runner {
        self.chroot = Some(chroot_path.as_ref().to_owned());
        self
    }

    /// Load the seccomp config before executing the program.
    pub fn seccomp(mut self, seccomp: seccomp::Context) -> Runner {
        self.seccomp = Some(seccomp);
        self
    }

    /// Add the process to the cgroup before it start running.
    pub fn cgroup(mut self, cgroup: cgroup::Context) -> Runner {
        self.cgroup = cgroup;
        self
    }

    /// Run the program and return the report of the process.
    ///
    /// Return `Error` if there is any trouble with running it.
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
        self.init_cgroup()?;
        self.cgroup.add_process(child)?;
        {
            let time_limit = self.resource_limit.real_time;
            thread::spawn(move || {
                thread::sleep(time_limit);
                nix::sys::signal::kill(child, nix::sys::signal::SIGKILL).unwrap_or_default();
            });
        }
        let exit_success = match nix::sys::wait::waitpid(child, None)? {
            nix::sys::wait::WaitStatus::Exited(_, code) => code == 0,
            nix::sys::wait::WaitStatus::Signaled(_, _, _) => false,
            _ => unreachable!("Should not appear other cases"),
        };
        let resource_usage = Resource::new(
            self.cgroup.cpuacct_controller().usage()?,
            start_time.elapsed(),
            self.cgroup.memory_controller().max_usage_in_bytes()?,
        );
        Ok(RunnerReport {
            exit_success,
            resource_usage,
        })
    }

    fn start_child(&self) -> Result<(), Box<dyn Error>> {
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

    fn init_cgroup(&self) -> io::Result<()> {
        let cpu_controller = self.cgroup.cpu_controller();
        let cpuacct_controller = self.cgroup.cpuacct_controller();
        let memory_controller = self.cgroup.memory_controller();
        cpu_controller.initialize()?;
        cpuacct_controller.initialize()?;
        memory_controller.initialize()?;
        let Resource {
            real_time,
            cpu_time,
            ..
        } = self.resource_limit;
        let period = Duration::from_secs(1);
        let quota = {
            let cpu_time = cpu_time.as_micros() as u32;
            let real_time = real_time.as_micros() as u32;
            period * cpu_time / real_time
        };
        cpu_controller.period().write(&period)?;
        cpu_controller.quota().write(&quota)?;
        memory_controller
            .limit_in_bytes()
            .write(&self.resource_limit.memory)?;
        Ok(())
    }

    fn init_seccomp(&self, program_ptr: *const i8) -> nix::Result<()> {
        if let Some(scmp) = &self.seccomp {
            let mut execve_whitelist =
                seccomp::Rule::whitelist(seccomp::Syscall::from_name("execve"));
            execve_whitelist.match_arg(seccomp::CmpOp::EQ, program_ptr as i64);
            scmp.add_rule(execve_whitelist)?;
            scmp.load()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
