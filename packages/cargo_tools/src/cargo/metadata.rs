use std::collections::HashMap;

use futures::future;
use itertools::Itertools;
use toml::Table;

use cargo_metadata::MetadataCommand;
pub use cargo_metadata::TargetKind;

use crate::cargo::Profile;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Metadata {
    packages: Vec<Package>,
    profiles: Vec<Profile>,
    target_dir: String,
}

impl Metadata {
    /// Returns the filenames of the manifests of the metadata's packages
    pub fn manifests(&self) -> Vec<String> {
        self.packages
            .iter()
            .map(|p| p.manifest.to_string())
            .collect()
    }

    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn target_dir(&self) -> &str {
        &self.target_dir
    }
}

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
    pub fn from_target(target: cargo_metadata::Target) -> Option<Self> {
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

    /// Counts the number of each [`Target`] kind in the given [`Package`]s.
    pub fn counts(packages: &[Package]) -> HashMap<Self, usize> {
        packages
            .iter()
            .flat_map(|p| p.targets.iter().map(|t| t.target_type))
            .counts()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("No Cargo toml found: {0}")]
    NoCargoToml(String),
    #[error(transparent)]
    Parse(cargo_metadata::Error),
}

struct CargoMetadata {
    packages: Vec<Package>,
    target_dir: String,
}

/// Tries to parse the metadata of a cargo project located at `root_dir`.
/// Executing necessary commands with `exec` and reading files with `read_file`.
pub async fn parse_metadata(
    root_dir: String,
    exec: impl AsyncFn(String, Vec<String>) -> Result<String, String>,
    read_file: impl AsyncFn(String) -> Result<String, String>,
) -> Result<Metadata, ParseError> {
    let (cargo_metadata, profiles) = future::join(
        parse_cargo_metadata(format!("{root_dir}/Cargo.toml"), exec),
        parse_profiles(root_dir, read_file),
    )
    .await;

    match cargo_metadata {
        Ok(CargoMetadata {
            packages,
            target_dir,
        }) => Ok(Metadata {
            packages,
            profiles,
            target_dir,
        }),
        Err(e) => Err(e),
    }
}

async fn parse_cargo_metadata(
    manifest_file: String,
    exec: impl AsyncFn(String, Vec<String>) -> Result<String, String>,
) -> Result<CargoMetadata, ParseError> {
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
    let metadata = exec("cargo".to_string(), cargo_args)
        .await
        .map_err(ParseError::NoCargoToml)?;

    let metadata = extract_raw_metadata(&metadata)?;

    let target_dir = metadata.target_directory.to_string();
    let packages = Package::from_metadata(metadata);

    Ok(CargoMetadata {
        packages,
        target_dir,
    })
}

async fn parse_profiles(
    root_dir: String,
    read_file: impl AsyncFn(String) -> Result<String, String>,
) -> Vec<Profile> {
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

    profiles
}

fn extract_raw_metadata(raw_metadata: &str) -> Result<cargo_metadata::Metadata, ParseError> {
    raw_metadata
        .lines()
        .find(|line| line.starts_with('{'))
        .ok_or(cargo_metadata::Error::NoJson)
        .and_then(MetadataCommand::parse)
        .map_err(ParseError::Parse)
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

/// The package information holding only information needed to build `cargo` commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Package {
    pub name: String,
    pub manifest: String,
    pub targets: Vec<Target>,
    pub features: Vec<String>,
}

impl Package {
    fn from_metadata(metadata: cargo_metadata::Metadata) -> Vec<Package> {
        metadata
            .packages
            .into_iter()
            .filter(|p| metadata.workspace_members.contains(&p.id))
            .map(Package::from_cargo)
            .sorted_by_key(|pkg| pkg.name.clone())
            .collect()
    }

    fn from_cargo(package: cargo_metadata::Package) -> Self {
        Self {
            name: package.name.to_string(),
            manifest: package.manifest_path.to_string(),
            targets: package
                .targets
                .into_iter()
                .filter_map(Target::try_from_cargo)
                .sorted_by_key(|t| t.target_type)
                .collect(),
            features: package.features.keys().cloned().collect(),
        }
    }
}

/// The condensed target information holding only information needed to build `cargo` commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Target {
    pub name: String,
    pub source: String,
    pub target_type: TargetType,
    pub target_kind: Vec<TargetKind>,
}

impl Target {
    pub fn try_from_cargo(target: cargo_metadata::Target) -> Option<Self> {
        let target_kind = target.kind.clone();
        Some(Self {
            name: target.name.to_string(),
            source: target.src_path.to_string(),
            target_type: TargetType::from_target(target)?,
            target_kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;
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

    /// Test successful metadata discovery from test-rust-project.
    #[wasm_bindgen_test(unsupported = test)]
    fn parse_valid_metadata() -> anyhow::Result<()> {
        let metadata = include_str!("../../res/test-rust-project-metadata.json").to_string();
        let metadata = extract_raw_metadata(&metadata)?;

        let packages = Package::from_metadata(metadata);

        // Expected packages from test-rust-project
        let expected_packages = vec![
            "core",
            "cli",
            "web-server",
            "utils",
            "test-cdylib",
            "test-staticlib",
            "test-proc-macro",
            "test-proc-macro-alt",
        ];

        check!(expected_packages.len() == packages.len());
        for expected in &expected_packages {
            check!(
                packages.iter().any(|pkg| &pkg.name == expected),
                "Expected package '{expected}' not found in: {:?}",
                packages
            );
        }

        Ok(())
    }
}
