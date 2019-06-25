//! Reduced high-level APIs for cgroup.
mod controller;
pub use controller::*;

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use nix;

pub struct Cgroup {
    name: String,
    controllers: HashSet<Controller>,
}
impl Cgroup {
    pub fn new() -> Cgroup {
        let name = uuid::Uuid::new_v4();
        Cgroup {
            name: name.to_string(),
            controllers: HashSet::new(),
        }
    }

    fn cgroup_path(&self, controller: Controller) -> Option<PathBuf> {
        if !self.controllers.contains(&controller) {
            return None;
        }
        Some(controller.cgroup_path().join(&self.name))
    }

    pub fn add_controller(&mut self, controller: Controller) -> io::Result<()> {
        if !self.controllers.contains(&controller) {
            self.controllers.insert(controller);
            fs::create_dir(self.cgroup_path(controller).unwrap())?;
        }
        Ok(())
    }

    pub fn add_process(&self, pid: nix::unistd::Pid) -> io::Result<()> {
        for controller in self.controllers.iter() {
            fs::write(
                self.cgroup_path(*controller).unwrap().join("cgroup.procs"),
                pid.to_string(),
            )?;
        }
        Ok(())
    }
}

impl Default for Cgroup {
    fn default() -> Cgroup {
        let mut cg = Cgroup::new();
        cg.add_controller(Controller::Cpu)
            .expect("Failed to add cpu controller");
        cg.add_controller(Controller::Memory)
            .expect("Failed to add memory controller");
        cg
    }
}

impl Cpu for Cgroup {
    fn cpu_usage(&self) -> io::Result<Duration> {
        let buf = fs::read(
            &self
                .cgroup_path(Controller::Cpu)
                .expect("Cpu controller is not exist")
                .join("cpuacct.usage"),
        )?;
        let time_ns = String::from_utf8(buf).unwrap().trim().parse().unwrap();
        Ok(Duration::from_nanos(time_ns))
    }

    fn set_cpu_period(&self, period: Duration) -> io::Result<()> {
        fs::write(
            &self
                .cgroup_path(Controller::Cpu)
                .expect("Cpu controller is not exist")
                .join("cpu.cfs_period_us"),
            period.as_micros().to_string(),
        )
    }

    fn set_cpu_quota(&self, quota: Duration) -> io::Result<()> {
        fs::write(
            &self
                .cgroup_path(Controller::Cpu)
                .expect("Cpu controller is not exist")
                .join("cpu.cfs_quota_us"),
            quota.as_micros().to_string(),
        )
    }
}

impl Memory for Cgroup {
    fn memory_usage(&self) -> io::Result<usize> {
        let buf = fs::read(
            &self
                .cgroup_path(Controller::Memory)
                .expect("Memory controller is not exist")
                .join("memory.max_usage_in_bytes"),
        )?;
        Ok(String::from_utf8(buf).unwrap().trim().parse().unwrap())
    }

    fn set_memory_limit(&self, limit: usize) -> io::Result<()> {
        fs::write(
            &self
                .cgroup_path(Controller::Memory)
                .expect("Memory controller is not exist")
                .join("memory.limit_in_bytes"),
            limit.to_string(),
        )
    }

    fn set_swappiness(&self, flag: bool) -> io::Result<()> {
        fs::write(
            &self
                .cgroup_path(Controller::Memory)
                .expect("Memory controller is not exist")
                .join("memory.swappiness"),
            flag.to_string(),
        )
    }

    fn memory_failcnt(&self) -> io::Result<usize> {
        let buf = fs::read(
            &self
                .cgroup_path(Controller::Memory)
                .expect("Memory controller is not exist")
                .join("memory.failcnt"),
        )?;
        Ok(String::from_utf8(buf).unwrap().trim().parse().unwrap())
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        for controller in self.controllers.iter() {
            fs::remove_dir(self.cgroup_path(*controller).unwrap())
                .expect("Failed to remove cgroup hierarchy");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;

    #[test]
    fn test_cgroup_path() {
        let ctx = Cgroup::default();

        let cpu_cgroup = ctx.cgroup_path(Controller::Cpu).unwrap();
        assert_eq!(cpu_cgroup, Controller::Cpu.cgroup_path().join(&ctx.name));
        assert_eq!(
            cpu_cgroup,
            PathBuf::from_iter(&["/sys/fs/cgroup/cpu/", ctx.name.as_str()])
        );
        assert!(cpu_cgroup.exists());

        let memory_cgroup = ctx.cgroup_path(Controller::Memory).unwrap();
        assert_eq!(
            memory_cgroup,
            Controller::Memory.cgroup_path().join(&ctx.name)
        );
        assert_eq!(
            memory_cgroup,
            PathBuf::from_iter(&["/sys/fs/cgroup/memory/", ctx.name.as_str()])
        );
        assert!(memory_cgroup.exists());
    }

    #[test]
    fn test_cpu_controller() -> io::Result<()> {
        let mut ctx = Cgroup::new();
        ctx.add_controller(Controller::Cpu)?;
        let cpu_cgroup = ctx.cgroup_path(Controller::Cpu).unwrap();

        ctx.set_cpu_period(Duration::from_millis(200))?;
        let mut cpu_period = fs::read(cpu_cgroup.join("cpu.cfs_period_us"))?;
        assert_eq!(cpu_period.pop(), Some(b'\n'));
        assert_eq!(
            cpu_period,
            Duration::from_millis(200)
                .as_micros()
                .to_string()
                .as_bytes()
        );

        ctx.set_cpu_period(Duration::from_millis(150))?;
        let mut cpu_period = fs::read(cpu_cgroup.join("cpu.cfs_period_us"))?;
        assert_eq!(cpu_period.pop(), Some(b'\n'));
        assert_eq!(
            cpu_period,
            Duration::from_millis(150)
                .as_micros()
                .to_string()
                .as_bytes()
        );

        ctx.set_cpu_quota(Duration::from_millis(500))?;
        let mut cpu_quota = fs::read(cpu_cgroup.join("cpu.cfs_quota_us"))?;
        assert_eq!(cpu_quota.pop(), Some(b'\n'));
        assert_eq!(
            cpu_quota,
            Duration::from_millis(500)
                .as_micros()
                .to_string()
                .as_bytes()
        );

        ctx.set_cpu_quota(Duration::from_millis(800))?;
        let mut cpu_quota = fs::read(cpu_cgroup.join("cpu.cfs_quota_us"))?;
        assert_eq!(cpu_quota.pop(), Some(b'\n'));
        assert_eq!(
            cpu_quota,
            Duration::from_millis(800)
                .as_micros()
                .to_string()
                .as_bytes()
        );

        let cpu_usage = ctx.cpu_usage()?;
        assert_eq!(cpu_usage, Duration::from_secs(0));

        Ok(())
    }

    #[test]
    fn test_memory_controller() -> io::Result<()> {
        let mut ctx = Cgroup::new();
        ctx.add_controller(Controller::Memory)?;
        let memory_cgroup = ctx.cgroup_path(Controller::Memory).unwrap();

        ctx.set_memory_limit(128 * 1024)?;
        let mut memory_limit = fs::read(memory_cgroup.join("memory.limit_in_bytes"))?;
        assert_eq!(memory_limit.pop(), Some(b'\n'));
        assert_eq!(memory_limit, (128 * 1024).to_string().as_bytes());

        ctx.set_memory_limit(256 * 1024)?;
        let mut memory_limit = fs::read(memory_cgroup.join("memory.limit_in_bytes"))?;
        assert_eq!(memory_limit.pop(), Some(b'\n'));
        assert_eq!(memory_limit, (256 * 1024).to_string().as_bytes());

        assert_eq!(ctx.memory_usage()?, 0);

        assert_eq!(ctx.memory_failcnt()?, 0);

        Ok(())
    }
}
