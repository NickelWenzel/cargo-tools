use std::collections::HashMap;

/// Gives the context in which a [Task] is run
pub enum CargoTask {
    Cargo(Task),
    CargoMake(Task),
    RustUp(Task),
}

/// Represents a task that can be executed e.g. on the command line
pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}
