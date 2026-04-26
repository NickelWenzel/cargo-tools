use cargo_tools::environment::Environment;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::vs_code_api;

const CARGO_TOOLS_SECTION: &str = "cargoTools";
const RUST_ANALYZER_SECTION: &str = "rust-analyzer";

/// The context in which a vs code task is run
/// Determines the environment e.g. if extra args are added for test or run etc.
#[derive(Debug, Clone, Copy)]
pub enum TaskContext {
    General,
    Run,
    Test,
}

/// Get the environment in which a VSCode task is run
/// This is done by accessing the VSCode configuration of the extension
/// by leveraging wasm bindgen and the VSCode API
pub fn environment(ctx: TaskContext) -> Environment {
    Environment {
        env: env(ctx),
        extra_args: extra_args(ctx),
        cargo_command: get(CARGO_TOOLS_SECTION, "cargoCommand", "cargo".to_string()),
    }
}

fn get<T: Serialize + DeserializeOwned + ToConfigValueType>(
    section: &str,
    key: &str,
    default: T,
) -> T {
    let value_type = T::to_config_value_type() as u32;
    let default_value = to_value(&default).unwrap_or(JsValue::NULL);
    let result = vs_code_api::get_config(section, key, value_type, default_value);
    from_value(result).unwrap_or(default)
}

#[wasm_bindgen]
pub enum ConfigValueType {
    Boolean,
    String,
    VecString,
    HashMapString,
}

trait ToConfigValueType {
    fn to_config_value_type() -> ConfigValueType;
}

impl ToConfigValueType for bool {
    fn to_config_value_type() -> ConfigValueType {
        ConfigValueType::Boolean
    }
}

impl ToConfigValueType for String {
    fn to_config_value_type() -> ConfigValueType {
        ConfigValueType::String
    }
}

impl ToConfigValueType for Vec<String> {
    fn to_config_value_type() -> ConfigValueType {
        ConfigValueType::VecString
    }
}

impl ToConfigValueType for HashMap<String, String> {
    fn to_config_value_type() -> ConfigValueType {
        ConfigValueType::HashMapString
    }
}

fn use_rust_analyzer_env_and_args() -> bool {
    get(CARGO_TOOLS_SECTION, "useRustAnalyzerEnvAndArgs", false)
}

fn env(ctx: TaskContext) -> HashMap<String, String> {
    let mut env = get(CARGO_TOOLS_SECTION, "extraEnv", HashMap::new());

    if use_rust_analyzer_env_and_args() {
        env.extend(get(RUST_ANALYZER_SECTION, "cargo.extraEnv", HashMap::new()));
    }

    match ctx {
        TaskContext::General => {}
        TaskContext::Run => {
            env.extend(get(CARGO_TOOLS_SECTION, "run.extraEnv", HashMap::new()));
        }
        TaskContext::Test => {
            env.extend(get(CARGO_TOOLS_SECTION, "test.extraEnv", HashMap::new()));
        }
    }

    env
}

fn extra_args(ctx: TaskContext) -> Vec<String> {
    let mut args = get(CARGO_TOOLS_SECTION, "buildArgs", Vec::new());

    if use_rust_analyzer_env_and_args() {
        args.extend(get(RUST_ANALYZER_SECTION, "cargo.extraArgs", Vec::new()));

        match ctx {
            TaskContext::General => {}
            TaskContext::Run => {
                args.extend(get(
                    RUST_ANALYZER_SECTION,
                    "runnables.extraArgs",
                    Vec::new(),
                ));
            }
            TaskContext::Test => {
                args.extend(get(
                    RUST_ANALYZER_SECTION,
                    "runnables.extraTestBinaryArgs",
                    Vec::new(),
                ));
            }
        }
    }

    match ctx {
        TaskContext::General => {}
        TaskContext::Run => {
            args.extend(get(CARGO_TOOLS_SECTION, "run.extraArgs", Vec::new()));
        }
        TaskContext::Test => {
            args.extend(get(CARGO_TOOLS_SECTION, "test.extraArgs", Vec::new()));
        }
    }

    args
}
