use std::collections::HashMap;

pub struct Environment {
    pub env: HashMap<String, String>,
    pub extra_args: Vec<String>,
    pub cargo_command: String,
}

impl Environment {
    pub fn new(
        env: HashMap<String, String>,
        extra_args: Vec<String>,
        cargo_command: String,
    ) -> Self {
        Self {
            env,
            extra_args,
            cargo_command,
        }
    }
}
