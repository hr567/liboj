//! Structures for online judge system.
use std::fmt::{self, Display};
use std::time::Duration;

/// Basic judge task.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub source: Source,
    pub problem: Problem,
}

/// Definition of some common problem types.
#[derive(Clone, Debug, PartialEq)]
pub enum Problem {
    Normal {
        limit: Resource,
        cases: Vec<TestCase>,
    },
    Special {
        limit: Resource,
        cases: Vec<TestCase>,
        spj: Source,
    },
}

impl Problem {
    /// Return the number of test cases in this problem.
    pub fn len(&self) -> usize {
        use Problem::*;
        match self {
            Normal { cases, .. } => cases.len(),
            Special { cases, .. } => cases.len(),
        }
    }

    /// Return `true` if the problem contains no test case.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Basic test case.
///
/// Only include a input content and a answer content.
#[derive(Clone, Debug, PartialEq)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

/// Basic source with the language and code.
///
/// The language is usually formatted into "{suffix}.{compiler}"
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub language: String,
    pub code: String,
}

/// Definition of all kinds of judge report.
#[derive(Clone, Debug, PartialEq)]
pub enum Report {
    Accepted { resource_usage: Resource },
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompileError,
    SystemError,
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Report::Accepted { resource_usage } => {
                writeln!(f, "Accepted:")?;
                writeln!(f, "{}", resource_usage)?;
            }
            Report::WrongAnswer => {
                writeln!(f, "Wrong Answer")?;
            }
            Report::TimeLimitExceeded => {
                writeln!(f, "Time Limit Exceeded")?;
            }
            Report::MemoryLimitExceeded => {
                writeln!(f, "Memory Limit Exceeded")?;
            }
            Report::RuntimeError => {
                writeln!(f, "Runtime Error")?;
            }
            Report::CompileError => {
                writeln!(f, "Compile Error")?;
            }
            Report::SystemError => {
                writeln!(f, "System Error")?;
            }
        }
        Ok(())
    }
}

/// Definition of resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resource {
    pub cpu_time: Duration,
    pub real_time: Duration,
    pub memory: usize, // in bytes
}

impl Resource {
    pub fn new(cpu_time: Duration, real_time: Duration, memory: usize) -> Resource {
        Resource {
            real_time,
            cpu_time,
            memory,
        }
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Real Time: {}", self.real_time.as_nanos())?;
        writeln!(f, "CPU Time: {}", self.cpu_time.as_nanos())?;
        writeln!(f, "Memory: {}", self.memory)?;
        Ok(())
    }
}
