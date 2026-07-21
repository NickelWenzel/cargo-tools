use std::collections::HashMap;

use itertools::Itertools;
use toml::Table;

use cargo_metadata::MetadataCommand;
pub use cargo_metadata::TargetKind;

use crate::{
    cargo::Profile,
    process::{CargoCommandEmpty, CargoTaskContext, Process},
};

/// Holds the [`Package`]s, [`Profile`]s and `target_dir` where cargo builds to.
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

    pub fn set_packages_and_target_dir(&mut self, packages_and_target_dir: PackagesAndTargetDir) {
        self.packages = packages_and_target_dir.packages;
        self.target_dir = packages_and_target_dir.target_dir;
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn set_profiles(&mut self, profiles: impl Into<Vec<Profile>>) {
        self.profiles = profiles.into()
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
    #[error("No Cargo toml found")]
    NoCargoToml,
    #[error("Failed to execute: {0}")]
    Exec(String),
    #[error(transparent)]
    Parse(cargo_metadata::Error),
    #[error(transparent)]
    CargoCommandEmpty(CargoCommandEmpty),
}

/// Holds the [`Package`]s and `target_dir` going into [`Metadata`].
#[derive(Debug, Clone)]
pub struct PackagesAndTargetDir {
    packages: Vec<Package>,
    target_dir: String,
}

/// Tries to parse the packages and target dir from `Cargo.toml` at `root_dir`.
/// Process Execution capabilities are client providedby `exec`.
pub async fn parse_packages_and_target_dir(
    manifest_file: String,
    ctx: CargoTaskContext,
    exec: impl AsyncFn(Process) -> Result<String, String>,
) -> Result<PackagesAndTargetDir, ParseError> {
    // Construct cargo metadata command with manifest path
    let args = vec![
        "metadata".to_string(),
        "--format-version".to_string(),
        "1".to_string(),
        "--manifest-path".to_string(),
        manifest_file,
        "--no-deps".to_string(),
    ];

    let process = ctx
        .try_into_process(args)
        .map_err(ParseError::CargoCommandEmpty)?;

    // Execute command via runtime
    let metadata = exec(process).await.map_err(ParseError::Exec)?;

    let metadata = extract_raw_metadata(&metadata)?;

    let target_dir = metadata.target_directory.to_string();
    let packages = Package::from_metadata(metadata);

    Ok(PackagesAndTargetDir {
        packages,
        target_dir,
    })
}

/// Tries to parse the profiles from the provided `file_paths`.
/// File reading capabilities are client provided by `read_file`.
pub async fn parse_profiles(
    file_paths: Vec<String>,
    read_file: impl AsyncFn(String) -> Result<String, String>,
) -> Vec<Profile> {
    let mut contents = Vec::new();
    for file in file_paths {
        contents.push(read_file(file).await);
    }

    let custom_profiles = contents
        .into_iter()
        .filter_map(Result::ok)
        .flat_map(extract_profiles);

    Profile::standards_profiles()
        .into_iter()
        .chain(custom_profiles)
        .unique()
        .collect()
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
            .sorted_by(|a, b| a.name.cmp(&b.name))
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
        check!(
            packages
                .windows(2)
                .all(|pair| pair[0].name.as_str() <= pair[1].name.as_str())
        );
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
