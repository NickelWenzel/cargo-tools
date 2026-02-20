use serde::{Deserialize, Serialize};

use crate::{
    cargo::selection,
    configuration::{Configuration, Context},
    runtime::{CargoTask, Task},
};

#[derive(Debug, Clone, Copy)]
pub enum Implicit {
    Build,
    Run,
    Debug,
    Test,
    Bench,
    Clean,
}

impl Implicit {
    pub fn to_task(self, selection: &selection::State, config: &impl Configuration) -> CargoTask {
        self.to_explicit(selection).to_task(selection, config)
    }

    pub fn to_explicit(self, selection: &selection::State) -> Explicit {
        match self {
            Implicit::Build => {
                let target = if let Some(package) = selection.package.clone() {
                    let target = selection.get(&package, |s| s.build_target.clone());
                    Some(BuildTarget { package, target })
                } else {
                    None
                };
                Explicit::Build(target)
            }
            Implicit::Run => {
                let target = if let Some(package) = selection.package.clone() {
                    let target = selection.get(&package, |s| s.run_target.clone());
                    Some(RunTarget { package, target })
                } else {
                    None
                };
                Explicit::Run(target)
            }
            Implicit::Debug => {
                let target = if let Some(package) = selection.package.clone() {
                    let target = selection.get(&package, |s| s.run_target.clone());
                    Some(RunTarget { package, target })
                } else {
                    None
                };
                Explicit::Debug(target)
            }
            Implicit::Test => {
                let package = selection.package.clone();
                Explicit::Test { package }
            }
            Implicit::Bench => {
                let package = selection.package.clone();
                Explicit::Bench { package }
            }
            Implicit::Clean => {
                let package = selection.package.clone();
                Explicit::Clean { package }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Explicit {
    Build(Option<BuildTarget>),
    Run(Option<RunTarget>),
    Debug(Option<RunTarget>),
    Test { package: Option<String> },
    Bench { package: Option<String> },
    Doc,
    Clean { package: Option<String> },
}

impl Explicit {
    pub fn to_task(self, selection: &selection::State, config: &impl Configuration) -> CargoTask {
        let ctx = self.task_context();

        let config_cmd = config.get_cargo_command(ctx);
        let mut cmd = config_cmd.split_whitespace().map(String::from);
        let (cmd, mut args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
        args.extend(self.into_args(selection));
        args.extend(config.get_extra_args(ctx));

        CargoTask::Cargo(Task {
            cmd,
            args,
            env: config.get_env(ctx),
        })
    }

    pub fn into_args(self, selection: &selection::State) -> Vec<String> {
        match self {
            Explicit::Build(build_target) => {
                let mut args = vec!["build".to_string()];
                let selection_args = selection.args(build_target.is_some());
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
            Explicit::Run(run_target) => {
                let mut args = vec!["run".to_string()];
                let selection_args = selection.args(run_target.is_some());
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
            Explicit::Debug(_) => Vec::new(),
            Explicit::Test { package } => {
                let mut args: Vec<_> = vec!["test".to_string()];
                let selection_args = selection.args(package.is_some());
                if let Some(package) = package {
                    args.extend(["--package".to_string(), package]);
                }
                args.extend(selection_args);
                args
            }
            Explicit::Bench { package } => {
                let mut args: Vec<_> = vec!["bench".to_string()];
                let selection_args = selection.args(package.is_some());
                if let Some(package) = package {
                    args.extend(["--package".to_string(), package]);
                }
                args.extend(selection_args);
                args
            }
            Explicit::Doc => ["doc", "--no-deps", "release"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            Explicit::Clean { package } => {
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

    fn task_context(&self) -> Context {
        match self {
            Explicit::Run(_) | Explicit::Debug(_) => Context::Run,
            Explicit::Test { package: _ } => Context::Test,
            Explicit::Build(_)
            | Explicit::Bench { package: _ }
            | Explicit::Doc
            | Explicit::Clean { package: _ } => Context::General,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub package: String,
    pub target: Option<BuildSubTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTarget {
    pub package: String,
    pub target: Option<RunSubTarget>,
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
}
