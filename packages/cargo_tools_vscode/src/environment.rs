use cargo_tools::{CargoCommand, process::CargoTaskContext};
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::vs_code_api;

const CARGO_TOOLS_SECTION: &str = "cargoTools";
const RUST_ANALYZER_SECTION: &str = "rust-analyzer";

fn general_task_context() -> CargoTaskContext {
    VsCodeTaskContext::General.to_cargo_task_context()
}

fn run_task_context() -> CargoTaskContext {
    VsCodeTaskContext::Run.to_cargo_task_context()
}

fn test_task_context() -> CargoTaskContext {
    VsCodeTaskContext::Test.to_cargo_task_context()
}

pub trait CommandExt {
    fn ctx(&self) -> CargoTaskContext;
}

impl CommandExt for CargoCommand {
    fn ctx(&self) -> CargoTaskContext {
        match self {
            Self::Run(_) | Self::Debug(_) => run_task_context(),
            Self::Test { package: _ } => test_task_context(),
            Self::Build(_) | Self::Bench(_) | Self::Doc | Self::Clean { package: _ } => {
                general_task_context()
            }
        }
    }
}

pub fn makefile_task_context() -> CargoTaskContext {
    general_task_context()
}

pub fn metadata_task_context() -> CargoTaskContext {
    general_task_context()
}

/// The context in which a vs code task is run
/// Determines the environment e.g. if extra args are added for test or run etc.
#[derive(Debug, Clone, Copy)]
enum VsCodeTaskContext {
    General,
    Run,
    Test,
}

impl VsCodeTaskContext {
    fn to_cargo_task_context(self) -> CargoTaskContext {
        CargoTaskContext::new(
            self.env(),
            self.extra_args(),
            get(CARGO_TOOLS_SECTION, "cargoCommand", "cargo".to_string()),
        )
    }

    fn env(&self) -> HashMap<String, String> {
        let mut env = get(CARGO_TOOLS_SECTION, "extraEnv", HashMap::new());

        if use_rust_analyzer_env_and_args() {
            env.extend(get(RUST_ANALYZER_SECTION, "cargo.extraEnv", HashMap::new()));
        }

        match self {
            Self::General => {}
            Self::Run => {
                env.extend(get(CARGO_TOOLS_SECTION, "run.extraEnv", HashMap::new()));
            }
            Self::Test => {
                env.extend(get(CARGO_TOOLS_SECTION, "test.extraEnv", HashMap::new()));
            }
        }

        env
    }

    fn extra_args(&self) -> Vec<String> {
        let mut args = get(CARGO_TOOLS_SECTION, "buildArgs", Vec::new());

        if use_rust_analyzer_env_and_args() {
            args.extend(get(RUST_ANALYZER_SECTION, "cargo.extraArgs", Vec::new()));

            match self {
                Self::General => {}
                Self::Run => {
                    args.extend(get(
                        RUST_ANALYZER_SECTION,
                        "runnables.extraArgs",
                        Vec::new(),
                    ));
                }
                Self::Test => {
                    args.extend(get(
                        RUST_ANALYZER_SECTION,
                        "runnables.extraTestBinaryArgs",
                        Vec::new(),
                    ));
                }
            }
        }

        match self {
            Self::General => {}
            Self::Run => {
                args.extend(get(CARGO_TOOLS_SECTION, "run.extraArgs", Vec::new()));
            }
            Self::Test => {
                args.extend(get(CARGO_TOOLS_SECTION, "test.extraArgs", Vec::new()));
            }
        }

        args
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
