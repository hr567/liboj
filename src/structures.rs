//! Structures for online judge system.
use std::fmt::{self, Display};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Definition of some common problem types.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

/// Basic judge task.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub source: Source,
    pub problem: Problem,
}

/// Basic test case.
/// Only include a input content and a answer content.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

/// Basic source with the language and code.
///
/// The language is usually formatted into "{suffix}.{compiler}"
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub language: String,
    pub code: String,
}

/// Definition of all kinds of judge report.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Report {
    Accepted {
        resource_usage: Resource,
    },
    WrongAnswer {
        resource_usage: Resource,
        message: String,
    },
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError {
        message: String,
    },
    CompileError {
        message: String,
    },
    SystemError {
        message: String,
    },
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Report::Accepted { resource_usage } => {
                writeln!(f, "Accepted:")?;
                writeln!(f, "{}", resource_usage)?;
            }
            Report::WrongAnswer {
                resource_usage,
                message,
            } => {
                writeln!(f, "Wrong Answer:")?;
                writeln!(f, "{}", resource_usage)?;
                writeln!(f, "Message: {}", message)?;
            }
            Report::TimeLimitExceeded => {
                writeln!(f, "Time Limit Exceeded")?;
            }
            Report::MemoryLimitExceeded => {
                writeln!(f, "Memory Limit Exceeded")?;
            }
            Report::RuntimeError { message } => {
                writeln!(f, "Runtime Error:")?;
                writeln!(f, "Message: {}", message)?;
            }
            Report::CompileError { message } => {
                writeln!(f, "Compile Error:")?;
                writeln!(f, "Message: {}", message)?;
            }
            Report::SystemError { message } => {
                writeln!(f, "System Error:")?;
                writeln!(f, "Message: {}", message)?;
            }
        }
        Ok(())
    }
}

/// Definition of resource.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
