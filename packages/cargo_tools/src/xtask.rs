use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::process::{CargoCommandEmpty, CargoTaskContext, Process};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnedAlias {
    pub name: String,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XtaskAlias {
    pub name: String,
    /// Normalized alias expansion, split into tokens.
    pub command: Vec<String>,
}

impl XtaskAlias {
    pub fn try_into_process(
        name: String,
        ctx: CargoTaskContext,
    ) -> Result<Process, CargoCommandEmpty> {
        // Run `cargo <alias-name>`; Cargo expands the alias at invocation time.
        ctx.try_into_process(vec![name])
    }

    pub fn try_into_process_with_extra_args(
        name: String,
        extra_args: Vec<String>,
        ctx: CargoTaskContext,
    ) -> Result<Process, CargoCommandEmpty> {
        let mut args = vec![name];
        args.extend(extra_args);
        ctx.try_into_process(args)
    }

    pub fn command_display(&self) -> String {
        self.command.join(" ")
    }

    fn keep(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        self.name.to_lowercase().contains(filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XtaskAliases(Vec<XtaskAlias>);

impl XtaskAliases {
    pub fn filtered(&self, filter: &str) -> Self {
        let filter = filter.to_lowercase();
        let aliases = self.iter().filter(|a| a.keep(&filter)).cloned().collect();
        Self(aliases)
    }
}

impl Deref for XtaskAliases {
    type Target = Vec<XtaskAlias>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for XtaskAliases {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<XtaskAlias>> for XtaskAliases {
    fn from(v: Vec<XtaskAlias>) -> Self {
        Self(v)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse .cargo/config.toml: {0}")]
    InvalidToml(String),
}

#[derive(Deserialize)]
struct CargoConfig {
    #[serde(default)]
    alias: HashMap<String, AliasValue>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AliasValue {
    String(String),
    Vec(Vec<String>),
}

impl AliasValue {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::String(s) => s.split_whitespace().map(String::from).collect(),
            Self::Vec(v) => v,
        }
    }
}

pub fn parse_config(config_content: &str) -> Result<XtaskAliases, ParseError> {
    let config: CargoConfig =
        toml::from_str(config_content).map_err(|e| ParseError::InvalidToml(e.to_string()))?;

    let mut aliases: Vec<XtaskAlias> = config
        .alias
        .into_iter()
        .map(|(name, value)| XtaskAlias {
            name,
            command: value.into_vec(),
        })
        .collect();

    aliases.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(XtaskAliases(aliases))
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{XtaskAlias, parse_config};

    #[wasm_bindgen_test(unsupported = test)]
    fn parse_valid_config() {
        let config = include_str!("../res/test-cargo-config.toml");
        let aliases = parse_config(config).expect("parse should succeed");

        let expected: Vec<&str> = vec!["compile", "lint", "xt-pkg", "xt-test"];
        for name in &expected {
            assert!(
                aliases.iter().any(|a| a.name == *name),
                "expected alias '{name}' not found; got: {:?}",
                aliases.iter().map(|a| &a.name).collect::<Vec<_>>()
            );
        }

        // String alias normalized to vec
        let compile = aliases.iter().find(|a| a.name == "compile").unwrap();
        assert_eq!(
            compile.command,
            vec!["run", "--package", "xtask", "--", "compile"]
        );

        // Vec alias kept as-is
        let lint = aliases.iter().find(|a| a.name == "lint").unwrap();
        assert_eq!(
            lint.command,
            vec!["run", "--package", "xtask", "--", "lint"]
        );
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn parse_empty_config_returns_empty_aliases() {
        let config = "[target.wasm32-unknown-unknown]\nrunner = 'wasm-bindgen-test-runner'\n";
        let aliases = parse_config(config).expect("parse should succeed");
        assert!(aliases.is_empty());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn filtering_works() {
        let config = include_str!("../res/test-cargo-config.toml");
        let aliases = parse_config(config).unwrap();

        let filtered = aliases.filtered("XT");
        assert!(filtered.iter().all(|a| a.name.contains("xt")));
        assert!(filtered.len() < aliases.len());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn try_into_process_uses_alias_name_as_arg() {
        use crate::process::CargoTaskContext;
        use std::collections::HashMap;

        let ctx = CargoTaskContext::new(HashMap::new(), vec![], "cargo".to_string());
        let process = XtaskAlias::try_into_process("compile".to_string(), ctx).unwrap();
        assert_eq!(process.cmd(), "cargo");
        assert_eq!(process.args(), &["compile"]);
    }
}
