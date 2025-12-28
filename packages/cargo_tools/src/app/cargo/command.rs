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
                if let Some(BuildTarget { package, target }) = build_target {
                    args.push("--package".to_string());
                    args.push(package);

                    if let Some(target) = target {
                        match target {
                            BuildSubTarget::Bin(bin) => {
                                args.push("--bin".to_string());
                                args.push(bin);
                            }
                            BuildSubTarget::Example(example) => {
                                args.push("--example".to_string());
                                args.push(example);
                            }
                            BuildSubTarget::Lib(_) => args.push("--lib".to_string()),
                            BuildSubTarget::Bench(bench) => {
                                args.push("--bench".to_string());
                                args.push(bench);
                            }
                        }
                    }
                }
                selection.append_platform_and_target(args)
            }
            Explicit::Run(run_target) => {
                let mut args = vec!["run".to_string()];
                if let Some(RunTarget { package, target }) = run_target {
                    args.push("--package".to_string());
                    args.push(package);

                    if let Some(target) = target {
                        match target {
                            RunSubTarget::Bin(bin) => {
                                args.push("--bin".to_string());
                                args.push(bin);
                            }
                            RunSubTarget::Example(example) => {
                                args.push("--example".to_string());
                                args.push(example);
                            }
                        }
                    }
                }
                selection.append_platform_and_target(args)
            }
            Explicit::Test { package } => {
                let mut args: Vec<_> = vec!["test".to_string()];
                if let Some(package) = package {
                    args.push("--package".to_string());
                    args.push(package);
                }
                selection.append_platform_and_target(args)
            }
            Explicit::Bench { package } => {
                let mut args: Vec<_> = vec!["bench".to_string()];
                if let Some(package) = package {
                    args.push("--package".to_string());
                    args.push(package);
                }
                selection.append_platform_and_target(args)
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
