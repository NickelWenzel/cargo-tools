use serde::{Deserialize, Serialize};

use crate::{
    cargo::{Config, metadata::Target},
    environment::Environment,
    runtime::{CargoTask, Task},
};

/// Represents a cargo command which can be executed as a [CargoTask]
#[derive(Debug, Clone)]
pub enum Command {
    Build(Option<BuildTarget>),
    Run(Option<RunTarget>),
    Debug(Option<RunTarget>),
    Test { package: Option<String> },
    Bench(Option<BenchTarget>),
    Doc,
    Clean { package: Option<String> },
}

impl Command {
    pub fn into_task(self, selection: &Config, environment: Environment) -> CargoTask {
        let Environment {
            env,
            extra_args,
            cargo_command,
        } = environment;

        let mut cmd = cargo_command.split_whitespace().map(String::from);
        let (cmd, mut args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
        args.extend(self.into_args(selection));
        args.extend(extra_args);

        CargoTask::Cargo(Task { cmd, args, env })
    }

    fn into_args(self, selection: &Config) -> Vec<String> {
        match self {
            Command::Build(build_target) => {
                let mut args = vec!["build".to_string()];
                let selection_args =
                    selection.args(build_target.as_ref().map(|t| t.package.as_str()));
                if let Some(BuildTarget { package, target }) = build_target {
                    args.extend(["--package".to_string(), package]);

                    if let Some(target) = target {
                        match target {
                            BuildSubTarget::Bin(bin) => {
                                args.extend(["--bin".to_string(), bin]);
                            }
                            BuildSubTarget::Example(example) => {
                                args.extend(["--example".to_string(), example]);
                            }
                            BuildSubTarget::Lib(_) => args.push("--lib".to_string()),
                            BuildSubTarget::Bench(bench) => {
                                args.extend(["--bench".to_string(), bench]);
                            }
                        }
                    }
                }
                args.extend(selection_args);
                args
            }
            Command::Run(run_target) => {
                let mut args = vec!["run".to_string()];
                let selection_args =
                    selection.args(run_target.as_ref().map(|t| t.package.as_str()));
                if let Some(RunTarget { package, target }) = run_target {
                    args.extend(["--package".to_string(), package]);

                    if let Some(target) = target {
                        match target {
                            RunSubTarget::Bin(bin) => {
                                args.extend(["--bin".to_string(), bin]);
                            }
                            RunSubTarget::Example(example) => {
                                args.extend(["--example".to_string(), example]);
                            }
                        }
                    }
                }
                args.extend(selection_args);
                args
            }
            // Debug is special and does not translate into arguments
            Command::Debug(_) => Vec::new(),
            Command::Test { package } => {
                let mut args: Vec<_> = vec!["test".to_string()];
                let selection_args = selection.args(package.as_deref());
                if let Some(package) = package {
                    args.extend(["--package".to_string(), package]);
                }
                args.extend(selection_args);
                args
            }
            Command::Bench(bench_target) => {
                let mut args = vec!["bench".to_string()];
                let selection_args =
                    selection.args(bench_target.as_ref().map(|t| t.package.as_str()));
                if let Some(BenchTarget { package, target }) = bench_target {
                    args.extend(["--package".to_string(), package]);

                    if let Some(target) = target {
                        args.push(target);
                    }
                }
                args.extend(selection_args);
                args
            }
            Command::Doc => ["doc", "--no-deps", "release"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            Command::Clean { package } => {
                let mut args = vec!["clean".to_string()];
                if let Some(package) = package {
                    args.extend(["--package".to_string(), package]);
                }
                if let Some(platform) = selection.platform_target.clone() {
                    args.extend(["--target".to_string(), platform]);
                }
                args.extend(selection.profile.cargo_args());
                args
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub package: String,
    pub target: Option<BuildSubTarget>,
}

impl BuildTarget {
    pub fn package_only(package: String) -> Self {
        Self {
            package,
            target: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTarget {
    pub package: String,
    pub target: Option<RunSubTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchTarget {
    pub package: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuildSubTarget {
    Bin(String),
    Example(String),
    Lib(String),
    Bench(String),
}

impl BuildSubTarget {
    pub fn name(&self) -> &str {
        match self {
            BuildSubTarget::Bin(name) => name,
            BuildSubTarget::Example(name) => name,
            BuildSubTarget::Lib(name) => name,
            BuildSubTarget::Bench(name) => name,
        }
    }

    pub fn matches(&self, target: Target, name: &str) -> bool {
        match self {
            BuildSubTarget::Bin(t) => target == Target::Bin && t == name,
            BuildSubTarget::Example(t) => target == Target::Example && t == name,
            BuildSubTarget::Lib(t) => target == Target::Lib && t == name,
            BuildSubTarget::Bench(t) => target == Target::Bench && t == name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunSubTarget {
    Bin(String),
    Example(String),
}

impl RunSubTarget {
    pub fn name(&self) -> &str {
        match self {
            RunSubTarget::Bin(name) => name,
            RunSubTarget::Example(name) => name,
        }
    }

    pub fn matches(&self, target: Target, name: &str) -> bool {
        match self {
            RunSubTarget::Bin(t) => target == Target::Bin && t == name,
            RunSubTarget::Example(t) => target == Target::Example && t == name,
        }
    }
}
