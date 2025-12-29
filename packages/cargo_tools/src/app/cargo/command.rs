use serde::{Deserialize, Serialize};

use crate::app::cargo::selection;

pub enum Implicit {
    Build,
    Run,
    Test,
    Bench,
    Doc,
}

impl Implicit {
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
            Implicit::Test => {
                let package = selection.package.clone();
                Explicit::Test { package }
            }
            Implicit::Bench => {
                let package = selection.package.clone();
                Explicit::Bench { package }
            }
            Implicit::Doc => Explicit::Doc,
        }
    }
}

pub enum Explicit {
    Build(Option<BuildTarget>),
    Run(Option<RunTarget>),
    Test { package: Option<String> },
    Bench { package: Option<String> },
    Doc,
}

impl Explicit {
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
        }
    }
}

pub struct BuildTarget {
    pub package: String,
    pub target: Option<BuildSubTarget>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunSubTarget {
    Bin(String),
    Example(String),
}
