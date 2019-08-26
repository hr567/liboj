// TODO: Add tests
//! Run a program in a new container with resource limit and system calls filter.

pub mod cgroup;
pub mod seccomp;

use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};
use std::sync::mpsc;
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
}

impl CommandExt for Command {
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
                .expect("Failed to unshare namespace");
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

trait ChildExt {
    fn timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus>;
    fn timeout_with_output(self, timeout: Duration) -> io::Result<Output>;
}

impl ChildExt for Child {
    fn timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus> {
        let (tx, rx) = mpsc::channel();
        let pid = nix::unistd::Pid::from_raw(self.id() as nix::libc::pid_t);
        thread::spawn(move || {
            match rx.recv_timeout(timeout) {
                Ok(_) => {} // Do nothing if the child process exit before times out
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Kill the child process if it times out
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Channel disconnected unexpected")
                }
            }
        });
        let status = self.wait()?;
        let _ = tx.send(());
        Ok(status)
    }

    fn timeout_with_output(self, timeout: Duration) -> io::Result<Output> {
        let (tx, rx) = mpsc::channel();
        let pid = nix::unistd::Pid::from_raw(self.id() as nix::libc::pid_t);
        thread::spawn(move || {
            match rx.recv_timeout(timeout) {
                Ok(_) => {} // Do nothing if the child process exit before times out
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Kill the child process if it times out
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Channel disconnected unexpected")
                }
            }
        });
        let output = self.wait_with_output()?;
        let _ = tx.send(());
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
