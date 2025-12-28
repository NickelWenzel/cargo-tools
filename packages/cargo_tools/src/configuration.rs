use std::collections::HashMap;

pub trait Configuration {
    fn get_env(&self, context: Context) -> HashMap<String, String>;
    fn get_extra_args(&self, config_context: Context) -> Vec<String>;
    fn get_cargo_command(&self, config_context: Context) -> String;
}

#[derive(Debug, Clone, Copy)]
pub enum Context {
    General,
    Run,
    Test,
}
