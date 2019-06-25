use super::*;

use std::fs;
use std::io::{self, prelude::*};
use std::time::Duration;

use tempfile;

const PROGRAM: &str = r#"/bin/sh"#;
const INPUT_CONTENT: &str = r#"echo -n "hello, world""#;
const ANSWER_CONTENT: &str = r#"hello, world"#;

fn generate_input_file() -> io::Result<tempfile::TempPath> {
    let mut input_file = tempfile::Builder::new().suffix(".in").tempfile()?;
    input_file.write_all(INPUT_CONTENT.as_bytes())?;
    Ok(input_file.into_temp_path())
}

fn generate_output_file() -> io::Result<tempfile::TempPath> {
    let output_file = tempfile::Builder::new().suffix(".out").tempfile()?;
    Ok(output_file.into_temp_path())
}

#[test]
fn test_basic_runner() -> io::Result<()> {
    let input_file = generate_input_file()?;
    let output_file = generate_output_file()?;
    let report = Runner::new(&PROGRAM, &input_file, &output_file)
        .run()
        .unwrap();
    assert!(report.exit_success);
    assert!(report.resource_usage.is_none());
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}

#[test]
fn test_runner_with_cgroup() -> io::Result<()> {
    let input_file = generate_input_file()?;
    let output_file = generate_input_file()?;
    let cg = Cgroup::default();
    let report = Runner::new(&PROGRAM, &input_file, &output_file)
        .cgroup(cg)
        .run()
        .unwrap();
    assert!(report.exit_success);
    assert!(report.resource_usage.is_some());
    let resource_usage = report.resource_usage.unwrap();
    assert_ne!(resource_usage.cpu_time_usage, Duration::from_secs(0));
    assert_ne!(resource_usage.real_time_usage, Duration::from_secs(0));
    assert_ne!(resource_usage.memory_usage, 0);
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}

#[test]
fn test_runner_with_chroot() -> io::Result<()> {
    let input_file = generate_input_file()?;
    let output_file = generate_input_file()?;
    let report = Runner::new(&PROGRAM, &input_file, &output_file)
        .chroot("/")
        .run()
        .unwrap();
    assert!(report.exit_success);
    assert!(report.resource_usage.is_none());
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}
