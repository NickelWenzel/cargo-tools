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

pub async fn parse_manual<RT: Runtime>(root_dir: String) -> MetadataUpdate {
    let mut profiles = Profile::standards_profiles();

    let manifest_file = format!("{root_dir}/Cargo.toml");
    let config_file = format!("{root_dir}/.cargo/Config.toml");

    for file in [manifest_file, config_file] {
        for profile in extract_custom_profiles::<RT>(file).await {
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

async fn extract_custom_profiles<RT: Runtime>(toml_path: String) -> Vec<Profile> {
    let Ok(toml) = RT::read_file(toml_path).await else {
        return Vec::new();
    };

    let Ok(table) = toml.parse::<Table>() else {
        return Vec::new();
    };

    table
        .keys()
        .filter(|k| k.starts_with("profile."))
        .map(|k| Profile::from(k.trim_start_matches("profile.")))
        .filter(Profile::is_standard)
        .collect::<Vec<_>>()
}
