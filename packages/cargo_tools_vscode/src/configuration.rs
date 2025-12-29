use cargo_tools::configuration;
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use wasm_bindgen::JsValue;

use crate::{contributes::ConfigPropertyType, vs_code_api};

const CARGO_TOOLS_SECTION: &str = "cargoTools";
const RUST_ANALYZER_SECTION: &str = "rust-analyzer";

pub struct Configuration;

impl Configuration {
    fn get<T: Serialize + DeserializeOwned>(section: &str, key: &str, default: T) -> T {
        let config_type = ConfigPropertyType::String as u32;
        let default_value = to_value(&default).unwrap_or(JsValue::NULL);
        let result = vs_code_api::get_config(section, key, config_type, default_value);
        from_value(result).unwrap_or_else(|_| default)
    }

    fn use_rust_analyzer_env_and_args() -> bool {
        Self::get(CARGO_TOOLS_SECTION, "useRustAnalyzerEnvAndArgs", false)
    }
}

impl configuration::Configuration for Configuration {
    fn get_env(&self, context: configuration::Context) -> HashMap<String, String> {
        let mut env = Self::get(CARGO_TOOLS_SECTION, "extraEnv", HashMap::new());

        if Self::use_rust_analyzer_env_and_args() {
            env.extend(Self::get(
                RUST_ANALYZER_SECTION,
                "cargo.extraEnv",
                HashMap::new(),
            ));
        }

        match context {
            configuration::Context::General => {}
            configuration::Context::Run => {
                env.extend(Self::get(
                    CARGO_TOOLS_SECTION,
                    "run.extraEnv",
                    HashMap::new(),
                ));
            }
            configuration::Context::Test => {
                env.extend(Self::get(
                    CARGO_TOOLS_SECTION,
                    "test.extraEnv",
                    HashMap::new(),
                ));
            }
        }

        env
    }

    fn get_extra_args(&self, config_context: configuration::Context) -> Vec<String> {
        let mut args = Self::get(CARGO_TOOLS_SECTION, "buildArgs", Vec::new());

        if Self::use_rust_analyzer_env_and_args() {
            args.extend(Self::get(
                RUST_ANALYZER_SECTION,
                "cargo.extraArgs",
                Vec::new(),
            ));

            match config_context {
                configuration::Context::General => {}
                configuration::Context::Run => {
                    args.extend(Self::get(
                        RUST_ANALYZER_SECTION,
                        "runnables.extraArgs",
                        Vec::new(),
                    ));
                }
                configuration::Context::Test => {
                    args.extend(Self::get(
                        RUST_ANALYZER_SECTION,
                        "runnables.extraTestBinaryArgs",
                        Vec::new(),
                    ));
                }
            }
        }

        match config_context {
            configuration::Context::General => {}
            configuration::Context::Run => {
                args.extend(Self::get(CARGO_TOOLS_SECTION, "run.extraArgs", Vec::new()));
            }
            configuration::Context::Test => {
                args.extend(Self::get(CARGO_TOOLS_SECTION, "test.extraArgs", Vec::new()));
            }
        }

        args
    }

    fn get_cargo_command(&self, _config_context: configuration::Context) -> String {
        Self::get(CARGO_TOOLS_SECTION, "cargoCommand", "cargo".to_string())
    }
}
