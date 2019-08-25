// TODO: Add tests
//! Run a program in a new container with resource limit and system calls filter.

pub mod cgroup;
pub mod seccomp;

use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use nix;

/// Extra features make Command run in a new container.
trait CommandExt {
    /// Load the seccomp config in child process.
    fn seccomp(&mut self, ctx: seccomp::Context) -> &mut Command;

    /// Attach the child process to the cgroup.
    fn cgroup(&mut self, ctx: cgroup::Context) -> &mut Command;

    /// Run program with all namespaces unshared.
    fn unshare_all_ns(&mut self) -> &mut Command;

    /// Chroot to a new path before exec.
    fn chroot<P: AsRef<Path>>(&mut self, new_root: P) -> &mut Command;

    /// Run the command with a time limit.
    /// If the command times out, the child process will be killed.
    fn timeout(&mut self, timeout: Duration) -> io::Result<Child>;
}

impl CommandExt for Command {
    fn timeout(&mut self, timeout: Duration) -> io::Result<Child> {
        let child = self.spawn()?;
        let pid = nix::unistd::Pid::from_raw(child.id() as nix::libc::pid_t);
        thread::spawn(move || {
            thread::sleep(timeout);
            let _ = nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL);
        });
        Ok(child)
    }

    fn seccomp(&mut self, ctx: seccomp::Context) -> &mut Command {
        unsafe {
            self.pre_exec(move || {
                ctx.load().expect("Failed to load seccomp context");
                Ok(())
            });
        }
        self
    }

    fn cgroup(&mut self, ctx: cgroup::Context) -> &mut Command {
        // Ensure that the cgroup context will not be dropped in child process
        let ctx = Box::leak(Box::new(ctx));
        unsafe {
            self.pre_exec(move || {
                ctx.add_process(nix::unistd::Pid::this())?;
                Ok(())
            });
        }
        self
    }

    fn unshare_all_ns(&mut self) -> &mut Command {
        unsafe {
            self.pre_exec(move || {
                nix::sched::unshare(nix::sched::CloneFlags::all())
                    .expect("Failed to unshare namespace");
                // Namespaces unshared in before version:
                // nix::sched::CloneFlags::empty()
                //     | nix::sched::CloneFlags::CLONE_FILES
                //     | nix::sched::CloneFlags::CLONE_FS
                //     | nix::sched::CloneFlags::CLONE_NEWCGROUP
                //     | nix::sched::CloneFlags::CLONE_NEWIPC
                //     | nix::sched::CloneFlags::CLONE_NEWNET
                //     | nix::sched::CloneFlags::CLONE_NEWNS
                //     | nix::sched::CloneFlags::CLONE_NEWPID
                //     | nix::sched::CloneFlags::CLONE_NEWUSER
                //     | nix::sched::CloneFlags::CLONE_NEWUTS
                //     | nix::sched::CloneFlags::CLONE_SYSVSEM,
                Ok(())
            });
        }
        self
    }

    fn chroot<P: AsRef<Path>>(&mut self, new_root: P) -> &mut Command {
        let new_root = new_root.as_ref().to_owned();
        unsafe {
            self.pre_exec(move || {
                nix::unistd::chroot(&new_root).expect("Failed to chroot to new path");
                nix::unistd::chdir("/").expect("Failed to chdir to new root");
                Ok(())
            });
        }
        self
    }
}

#[cfg(test)]
mod tests;
