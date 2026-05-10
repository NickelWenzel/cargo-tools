use std::collections::HashMap;

use itertools::Itertools;
use toml::Table;

use cargo_metadata::{Metadata, MetadataCommand, TargetKind};

use crate::cargo::Profile;

/// Represents the kinds of targets which a `cargo` command can target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TargetType {
    Lib,
    Bin,
    Example,
    Bench,
}

impl TargetType {
    /// Converts a [`cargo_metadata::Target`] into a [`Target`].
    pub fn from_target(target: &cargo_metadata::Target) -> Option<Self> {
        target.kind.iter().find_map(|kind| match kind {
            TargetKind::Lib
            | TargetKind::RLib
            | TargetKind::DyLib
            | TargetKind::CDyLib
            | TargetKind::StaticLib
            | TargetKind::ProcMacro => Some(Self::Lib),
            TargetKind::Bin => Some(Self::Bin),
            TargetKind::Example => Some(Self::Example),
            TargetKind::Bench => Some(Self::Bench),
            // Not yet supported
            TargetKind::Test | TargetKind::CustomBuild | TargetKind::Unknown(_) => None,
            // enum is non-exhaustive
            _ => None,
        })
    }

    /// Counts the number of each [`Target`] kind in the given [`cargo_metadata::Metadata`].
    pub fn counts(packages: &[CondensedPackage]) -> HashMap<Self, usize> {
        packages
            .iter()
            .flat_map(|p| p.targets.iter().map(|t| t.target_type))
            .counts()
    }
}

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    Metadata(Metadata),
    Profiles(Vec<Profile>),
    NoCargoToml(String),
    FailedToRetrieve,
}

/// Returns the filenames of the manifests of the metadata's workspace packages
pub fn workspace_manifests(metadata: &Metadata) -> Vec<String> {
    metadata
        .workspace_packages()
        .into_iter()
        .map(|p| p.manifest_path.to_string())
        .collect()
}

/// Parses the metadata and returns a metadata update
pub async fn parse_metadata(
    manifest_file: String,
    exec: impl AsyncFn(String, Vec<String>) -> Result<String, String>,
) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let cargo_args = vec![
        "metadata".to_string(),
        "--format-version".to_string(),
        "1".to_string(),
        "--manifest-path".to_string(),
        manifest_file,
        "--no-deps".to_string(),
    ];

    // Execute command via runtime
    match exec("cargo".to_string(), cargo_args).await {
        Ok(metadata) => extract_raw_metadata(&metadata).await,
        Err(e) => MetadataUpdate::NoCargoToml(e),
    }
}

/// Parses the metadata for profiles and returns a metadata update
pub async fn parse_profiles(
    root_dir: String,
    read_file: impl AsyncFn(String) -> Result<String, String>,
) -> MetadataUpdate {
    let mut profiles = Profile::standards_profiles();

    let manifest_file = format!("{root_dir}/Cargo.toml");
    let config_file = format!("{root_dir}/.cargo/Config.toml");

    for file in [manifest_file, config_file] {
        let Ok(toml) = read_file(file).await else {
            continue;
        };
        for profile in extract_profiles(toml) {
            if !profiles.contains(&profile) {
                profiles.push(profile);
            }
        }
    }

    MetadataUpdate::Profiles(profiles)
}

async fn extract_raw_metadata(raw_metadata: &str) -> MetadataUpdate {
    let Some(metadata) = raw_metadata.lines().find(|line| line.starts_with('{')) else {
        return MetadataUpdate::FailedToRetrieve;
    };

    // Parse JSON output into Metadata
    match MetadataCommand::parse(metadata) {
        Ok(metadata) => MetadataUpdate::Metadata(metadata),
        Err(e) => MetadataUpdate::NoCargoToml(e.to_string()),
    }
}

fn extract_profiles(toml: String) -> Vec<Profile> {
    let Ok(table) = toml.parse::<Table>() else {
        return Vec::new();
    };
    let Some(profiles) = table.get("profile") else {
        return Vec::new();
    };

    let profiles = match profiles {
        toml::Value::Table(profiles) => profiles,
        _ => return Vec::new(),
    };

    profiles.keys().cloned().map(Profile::from).collect()
}

/// The condensed package information holding only information needed to build `cargo` commands
#[derive(Debug)]
pub struct CondensedPackage {
    pub name: String,
    pub manifest: String,
    pub targets: Vec<CondensedTarget>,
    pub features: Vec<String>,
}

/// The condensed target information holding only information needed to build `cargo` commands
#[derive(Debug)]
pub struct CondensedTarget {
    pub name: String,
    pub source: String,
    pub target_type: TargetType,
    pub original_types: Vec<TargetKind>,
}

impl CondensedTarget {
    pub fn try_from_cargo(target: &cargo_metadata::Target) -> Option<Self> {
        Some(Self {
            name: target.name.to_string(),
            source: target.src_path.to_string(),
            target_type: TargetType::from_target(target)?,
            original_types: target.kind.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn extract_profiles_valid() {
        let toml = r#"
[workspace]
members = ["cli", "core", "utils", "web-server", "test-cdylib", "test-staticlib", "test-proc-macro", "test-proc-macro-alt"]

[workspace.package]
version = "0.1.0"
edition = "2021"

[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3

[profile.debug-optimized]
inherits = "dev"
opt-level = 2
        "#;

        let profiles = extract_profiles(toml.to_string());

        assert_eq!(profiles.len(), 3);
        assert!(profiles.contains(&Profile::from("debug-optimized")));
        assert!(profiles.contains(&Profile::from("dev")));
        assert!(profiles.contains(&Profile::from("release")));
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn extract_profiles_empty() {
        let toml = r#"
[package]
name = "test"
version = "0.1.0"
        "#;

        let profiles = extract_profiles(toml.to_string());
        assert_eq!(profiles.len(), 0);
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn extract_profiles_invalid_toml() {
        let toml = "not valid toml {{{";

        let profiles = extract_profiles(toml.to_string());
        assert_eq!(profiles.len(), 0);
    }
}
