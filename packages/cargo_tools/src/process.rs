use std::collections::HashMap;

/// Represents a task that can be executed e.g. on the command line
pub struct Process {
    cmd: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl Process {
    pub fn new(cmd: String, args: Vec<String>, env: HashMap<String, String>) -> Self {
        Self { cmd, args, env }
    }
    pub fn cmd(&self) -> &str {
        &self.cmd
    }
    pub fn args(&self) -> &[String] {
        &self.args
    }
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}

#[derive(Debug, Clone)]
pub struct CargoTaskContext {
    env: HashMap<String, String>,
    extra_args: Vec<String>,
    cargo_cmd: String,
}

impl CargoTaskContext {
    pub fn new(env: HashMap<String, String>, extra_args: Vec<String>, cargo_cmd: String) -> Self {
        Self {
            env,
            extra_args,
            cargo_cmd,
        }
    }

    pub fn try_into_process(self, args: Vec<String>) -> Result<Process, CargoCommandEmpty> {
        let Self {
            env,
            extra_args,
            cargo_cmd,
        } = self;

        let mut cmd_parts = cargo_cmd.split_whitespace();
        let cmd = cmd_parts.next().ok_or(CargoCommandEmpty)?.to_string();
        let args = cmd_parts
            .map(ToString::to_string)
            .chain(args)
            .chain(extra_args)
            .collect();

        Ok(Process { cmd, args, env })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The configured 'cargo' is empty")]
pub struct CargoCommandEmpty;
