use toml::Table;

use cargo_metadata::{Metadata, MetadataCommand};

use crate::{profile::Profile, runtime::Runtime};

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    Metadata(Metadata),
    Profiles(Vec<Profile>),
    NoCargoToml,
    FailedToRetrieve,
}

pub fn workspace_manifests(metadata: &Metadata) -> Vec<String> {
    metadata
        .workspace_packages()
        .into_iter()
        .map(|p| p.manifest_path.to_string())
        .collect()
}

pub async fn parse_metadata<RT: Runtime>(manifest_file: String) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let command =
        format!("cargo metadata --format-version 1 --manifest-path {manifest_file} --no-deps");

    // Execute command via runtime
    match RT::exec(command).await {
        Ok(metadata) => extract_raw_metadata::<RT>(&metadata).await,
        Err(e) => {
            RT::log(format!("Failed to generate cargo metadata: {e}"));
            MetadataUpdate::NoCargoToml
        }
    }
}

pub async fn parse_profiles<RT: Runtime>(root_dir: String) -> MetadataUpdate {
    let mut profiles = Profile::standards_profiles();

    let manifest_file = format!("{root_dir}/Cargo.toml");
    let config_file = format!("{root_dir}/.cargo/Config.toml");

    for file in [manifest_file, config_file] {
        let Ok(toml) = RT::read_file(file).await else {
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

async fn extract_raw_metadata<RT: Runtime>(raw_metadata: &str) -> MetadataUpdate {
    let Some(metadata) = raw_metadata.lines().find(|line| line.starts_with('{')) else {
        RT::log("Cargo metadata do not contain valid JSON".to_string());
        return MetadataUpdate::FailedToRetrieve;
    };

    // Parse JSON output into Metadata
    match MetadataCommand::parse(metadata) {
        Ok(metadata) => MetadataUpdate::Metadata(metadata),
        Err(e) => {
            RT::log(format!("Failed to parse cargo metadata: {e}"));
            MetadataUpdate::NoCargoToml
        }
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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

    #[test]
    fn extract_profiles_empty() {
        let toml = r#"
[package]
name = "test"
version = "0.1.0"
        "#;

        let profiles = extract_profiles(toml.to_string());
        assert_eq!(profiles.len(), 0);
    }

    #[test]
    fn extract_profiles_invalid_toml() {
        let toml = "not valid toml {{{";

        let profiles = extract_profiles(toml.to_string());
        assert_eq!(profiles.len(), 0);
    }
}
