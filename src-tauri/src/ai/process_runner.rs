use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRunSpec {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRunOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProcessRunErrorCode {
    SpawnFailed,
    IoFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRunError {
    pub code: ProcessRunErrorCode,
    pub message: String,
}

pub trait ProcessRunner {
    fn run(&self, spec: ProcessRunSpec) -> Result<ProcessRunOutput, ProcessRunError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn run(&self, spec: ProcessRunSpec) -> Result<ProcessRunOutput, ProcessRunError> {
        run_process(spec)
    }
}

impl ProcessRunError {
    fn spawn_failed(message: impl Into<String>) -> Self {
        Self {
            code: ProcessRunErrorCode::SpawnFailed,
            message: message.into(),
        }
    }

    fn io_failed(message: impl Into<String>) -> Self {
        Self {
            code: ProcessRunErrorCode::IoFailed,
            message: message.into(),
        }
    }

    fn timed_out(timeout: Duration) -> Self {
        Self {
            code: ProcessRunErrorCode::TimedOut,
            message: format!("process timed out after {} ms", timeout.as_millis()),
        }
    }
}

pub fn run_process(spec: ProcessRunSpec) -> Result<ProcessRunOutput, ProcessRunError> {
    let mut command = Command::new(&spec.program);
    command.args(&spec.args);
    if let Some(working_dir) = &spec.working_dir {
        command.current_dir(working_dir);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| ProcessRunError::spawn_failed(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProcessRunError::io_failed("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ProcessRunError::io_failed("failed to capture stderr"))?;
    let stdout_reader = thread::spawn(move || read_pipe(stdout));
    let stderr_reader = thread::spawn(move || read_pipe(stderr));
    let deadline = Instant::now() + spec.timeout;

    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| ProcessRunError::io_failed(error.to_string()))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(ProcessRunError::timed_out(spec.timeout));
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = stdout_reader
        .join()
        .map_err(|_| ProcessRunError::io_failed("failed to join stdout reader"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| ProcessRunError::io_failed("failed to join stderr reader"))??;

    Ok(ProcessRunOutput {
        stdout,
        stderr,
        exit_code: status.code(),
    })
}

fn read_pipe(mut pipe: impl Read) -> Result<String, ProcessRunError> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .map_err(|error| ProcessRunError::io_failed(error.to_string()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
