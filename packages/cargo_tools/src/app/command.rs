use serde::{Deserialize, Serialize};

pub enum Command {
    Cargo(CargoCommand),
    CargoMake { task: String },
}

impl Command {}

pub enum CargoCommand {
    Implicit(ImplicitCargoCommand),
    Explicit(ExplicitCargoCommand),
}

pub enum ImplicitCargoCommand {
    Build,
    Run,
    Test,
    Benchmark,
    Doc,
}

pub enum ExplicitCargoCommand {
    Build(Option<BuildTarget>),
    Run(RunTarget),
    Test { package: Option<String> },
    Benchmark { package: Option<String> },
    Doc,
}

pub struct BuildTarget {
    pub package: String,
    pub target: Option<BuildSubTarget>,
}

pub struct RunTarget {
    pub package: String,
    pub target: RunSubTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuildSubTarget {
    Bin(String),
    Example(String),
    Lib(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunSubTarget {
    Bin(String),
    Example(String),
}
