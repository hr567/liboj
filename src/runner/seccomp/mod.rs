//! Reduced high-level APIs for libseccomp.
mod libseccomp;
use libseccomp::*;

use std::ffi::CString;

use nix;

pub type Syscall = u32;
pub fn syscall_resolve_name(name: &str) -> Option<Syscall> {
    let name = CString::new(name).unwrap();
    let syscall = unsafe { seccomp_syscall_resolve_name(name.as_ptr()) };
    if syscall < 0 {
        None
    } else {
        Some(syscall as u32)
    }
}

pub struct ScmpCtx {
    ctx: scmp_filter_ctx,
}

impl Default for ScmpCtx {
    fn default() -> ScmpCtx {
        ScmpCtx {
            ctx: unsafe { seccomp_init(Act::Kill as u32) },
        }
    }
}

impl ScmpCtx {
    pub fn new(act: Act) -> ScmpCtx {
        ScmpCtx {
            ctx: unsafe { seccomp_init(act as u32) },
        }
    }

    pub fn add_rule(&self, act: Act, syscall: Syscall, pattern: Pattern) -> nix::Result<()> {
        let pattern = pattern.as_scmp_arg_cmp();

        let rc = unsafe {
            seccomp_rule_add_array(
                self.ctx,
                act as u32,
                syscall as i32,
                pattern.len() as u32,
                pattern.as_ptr(),
            )
        };

        if rc < 0 {
            return Err(nix::Error::from(nix::errno::from_i32(-rc)));
        }

        Ok(())
    }

    pub fn whitelist(&self, syscall: Syscall, pattern: Pattern) -> nix::Result<()> {
        self.add_rule(Act::Allow, syscall, pattern)
    }

    pub fn blacklist(&self, syscall: Syscall, pattern: Pattern) -> nix::Result<()> {
        self.add_rule(Act::Kill, syscall, pattern)
    }

    pub fn load(&self) -> Result<(), nix::errno::Errno> {
        if unsafe { seccomp_load(self.ctx) } == 0 {
            Ok(())
        } else {
            Err(nix::errno::from_i32(nix::errno::errno()))
        }
    }
}

impl Drop for ScmpCtx {
    fn drop(&mut self) {
        unsafe {
            seccomp_release(self.ctx);
        }
    }
}

pub struct Pattern(Vec<(CmpOp, i64)>);

impl Pattern {
    pub fn new() -> Pattern {
        Pattern::default()
    }

    pub fn add_arg(&mut self, op: CmpOp, arg: i64) {
        self.0.push((op, arg));
    }

    pub fn as_scmp_arg_cmp(&self) -> Vec<scmp_arg_cmp> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, (op, value))| scmp_arg_cmp {
                arg: index as u32,
                op: *op as u32,
                datum_a: *value as u64,
                datum_b: 0,
            })
            .collect()
    }
}

impl Default for Pattern {
    fn default() -> Pattern {
        Pattern(Vec::new())
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Act {
    Kill = SCMP_ACT_KILL,
    // KillProcess = SCMP_ACT_KILL_PROCESS,
    Trap = SCMP_ACT_TRAP,
    // Errno = SCMP_ACT_ERRNO,
    // Trace = SCMP_ACT_TRACE,
    Log = SCMP_ACT_LOG,
    Allow = SCMP_ACT_ALLOW,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CmpOp {
    // Min = scmp_compare__SCMP_CMP_MIN,
    NE = scmp_compare_SCMP_CMP_NE,
    LT = scmp_compare_SCMP_CMP_LT,
    LE = scmp_compare_SCMP_CMP_LE,
    EQ = scmp_compare_SCMP_CMP_EQ,
    GE = scmp_compare_SCMP_CMP_GE,
    GT = scmp_compare_SCMP_CMP_GT,
    // MaskedEq = scmp_compare_SCMP_CMP_MASKED_EQ,
    // Max = scmp_compare__SCMP_CMP_MAX,
}
