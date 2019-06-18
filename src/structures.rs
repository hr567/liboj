//! Structures for online judge system.
use std::fmt;
use std::time;

/// Definition of some common problem types.
#[derive(Debug)]
pub enum Problem {
    Normal {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
        test_cases: Vec<TestCase>,
    },
    Special {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
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
#[derive(Debug)]
pub struct JudgeTask {
    pub source: Source,
    pub problem: Problem,
}

/// Basic test case.
/// Only include a input content and a answer content.
#[derive(Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

/// Source code structure.
/// Use language to decide which compiler to use
#[derive(Debug)]
pub struct Source {
    pub language: String,
    pub code: String,
}

/// Report of judging result for each test case.
#[derive(Debug)]
pub struct JudgeReport {
    /// Index of the test case, should be started from 0
    pub index: usize,
    /// One of the judge results
    pub result: JudgeResult,
    /// Time usage of this test case
    pub time_usage: time::Duration,
    /// Memory usage of this test case(in Bytes)
    pub memory_usage: usize,
}

impl JudgeReport {
    pub fn new(
        index: usize,
        result: JudgeResult,
        time_usage: time::Duration,
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
#[derive(Debug)]
pub enum JudgeResult {
    CE,
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    RE,
}

impl fmt::Display for JudgeResult {
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
