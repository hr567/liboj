//! Structures for online judge system.
use std::fmt::{self, Display};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Definition of some common problem types.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Problem {
    Normal {
        limit: ResourceLimit,
        test_cases: Vec<TestCase>,
    },
    Special {
        limit: ResourceLimit,
        test_cases: Vec<TestCase>,
        spj: Source,
    },
}

impl Problem {
    /// Return the number of test cases in this problem.
    pub fn len(&self) -> usize {
        use Problem::*;
        match self {
            Normal { test_cases, .. } => test_cases.len(),
            Special { test_cases, .. } => test_cases.len(),
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
        resource_usage: ResourceUsage,
    },
    WrongAnswer {
        resource_usage: ResourceUsage,
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

/// Resource usage of the judged program.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_time_usage: Duration,
    pub real_time_usage: Duration,
    pub memory_usage: usize, // in bytes
}

impl ResourceUsage {
    pub fn new(
        cpu_time_usage: Duration,
        real_time_usage: Duration,
        memory_usage: usize,
    ) -> ResourceUsage {
        ResourceUsage {
            cpu_time_usage,
            real_time_usage,
            memory_usage,
        }
    }
}

impl Display for ResourceUsage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CPU time usage: {}", self.cpu_time_usage.as_nanos())?;
        writeln!(f, "Real time usage: {}", self.real_time_usage.as_nanos())?;
        writeln!(f, "Memory usage: {}", self.memory_usage)?;
        Ok(())
    }
}

/// Resource limit of the judged program.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimit {
    pub time_limit: Duration,
    pub memory_limit: usize, // in bytes
}

impl ResourceLimit {
    pub fn new(time_limit: Duration, memory_limit: usize) -> ResourceLimit {
        ResourceLimit {
            time_limit,
            memory_limit,
        }
    }
}
