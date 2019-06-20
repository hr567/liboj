//! Structures for online judge system.
use std::fmt::{self, Display};
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Definition of some common problem types.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Problem {
    Normal {
        time_limit: Duration,
        memory_limit: usize, // Bytes
        test_cases: Vec<TestCase>,
    },
    Special {
        time_limit: Duration,
        memory_limit: usize, // Bytes
        test_cases: Vec<TestCase>,
        spj: Source,
    },
}

impl Problem {
    /// The size of the test cases.
    pub fn len(&self) -> usize {
        use Problem::*;
        match self {
            Normal { test_cases, .. } => test_cases.len(),
            Special { test_cases, .. } => test_cases.len(),
        }
    }

    /// Is the problem without any test case.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Basic judge task structure.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeTask {
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

/// Source code structure.
/// Use language to decide which compiler to use
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub language: String,
    pub code: String,
}

/// Report of judging result for each test case.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeReport {
    /// Index of the test case, should be started from 0
    pub index: usize,
    /// One of the judge results
    pub result: JudgeResult,
    /// Time usage of this test case
    pub time_usage: Duration,
    /// Memory usage of this test case(in Bytes)
    pub memory_usage: usize,
}

impl JudgeReport {
    pub fn new(
        index: usize,
        result: JudgeResult,
        time_usage: Duration,
        memory_usage: usize,
    ) -> JudgeReport {
        JudgeReport {
            index,
            result,
            time_usage,
            memory_usage,
        }
    }
}

/// Definition of all judge results
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum JudgeResult {
    CE,
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    RE,
}

impl Display for JudgeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use JudgeResult::*;
        let s = match self {
            AC => "AC",
            CE => "CE",
            MLE => "MLE",
            OLE => "OLE",
            RE => "RE",
            TLE => "TLE",
            WA => "WA",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for JudgeResult {
    type Err = ();

    fn from_str(s: &str) -> Result<JudgeResult, ()> {
        match s {
            "AC" => Ok(JudgeResult::AC),
            "CE" => Ok(JudgeResult::CE),
            "MLE" => Ok(JudgeResult::MLE),
            "OLE" => Ok(JudgeResult::OLE),
            "RE" => Ok(JudgeResult::RE),
            "TLE" => Ok(JudgeResult::TLE),
            "WA" => Ok(JudgeResult::WA),
            _ => Err(()),
        }
    }
}
