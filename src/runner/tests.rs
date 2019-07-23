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
    let report = Runner::new(
        &PROGRAM,
        &input_file,
        &output_file,
        Resource::new(
            Duration::from_secs(2),
            Duration::from_secs(1),
            16 * 1024 * 1024,
        ),
    )
    .run()
    .unwrap();
    assert!(report.exit_success);
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}

#[test]
fn test_runner_with_cgroup() -> io::Result<()> {
    let input_file = generate_input_file()?;
    let output_file = generate_input_file()?;
    let cg = cgroup::Context::new();
    let report = Runner::new(
        &PROGRAM,
        &input_file,
        &output_file,
        Resource::new(
            Duration::from_secs(2),
            Duration::from_secs(1),
            16 * 1024 * 1024,
        ),
    )
    .cgroup(cg)
    .run()
    .unwrap();
    assert!(report.exit_success);
    let resource_usage = report.resource_usage;
    assert_ne!(resource_usage.cpu_time, Duration::from_secs(0));
    assert_ne!(resource_usage.real_time, Duration::from_secs(0));
    assert_ne!(resource_usage.memory, 0);
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}

#[test]
fn test_runner_with_chroot() -> io::Result<()> {
    let input_file = generate_input_file()?;
    let output_file = generate_input_file()?;
    let report = Runner::new(
        &PROGRAM,
        &input_file,
        &output_file,
        Resource::new(
            Duration::from_secs(2),
            Duration::from_secs(1),
            16 * 1024 * 1024,
        ),
    )
    .chroot("/")
    .run()
    .unwrap();
    assert!(report.exit_success);
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());
    Ok(())
}
