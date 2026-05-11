use serde::{Deserialize, Serialize};

const DEV: &str = "dev";
const RELEASE: &str = "release";
const TEST: &str = "test";
const BENCH: &str = "bench";
const DOC: &str = "doc";

/// Represents a Cargo profile, which can be either standard, custom, or none.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum Profile {
    /// No profile selection
    #[default]
    None,
    Dev,
    Release,
    Test,
    Bench,
    Doc,
    /// Custom user-defined profile
    Custom(String),
}

impl Profile {
    /// Returns the profile name as used in Cargo commands.
    pub fn get_name(&self) -> Option<&str> {
        match self {
            Profile::None => None,
            Profile::Dev => Some(DEV),
            Profile::Release => Some(RELEASE),
            Profile::Test => Some(TEST),
            Profile::Bench => Some(BENCH),
            Profile::Doc => Some(DOC),
            Profile::Custom(name) => Some(name.as_str()),
        }
    }

    /// Returns a human-readable display name for the profile.
    pub fn get_display_name(&self) -> &str {
        match self {
            Profile::None => "No selection",
            Profile::Dev => "Development",
            Profile::Release => "Release",
            Profile::Test => "Test",
            Profile::Bench => "Benchmark",
            Profile::Doc => "Documentation",
            Profile::Custom(name) => name.as_str(),
        }
    }

    /// Returns a description of the profile for use in UI elements.
    pub fn get_description(&self) -> String {
        match self {
            Profile::None => "No profile selected".to_string(),
            Profile::Dev => {
                "Development profile (default) with debugging symbols and no optimizations"
                    .to_string()
            }
            Profile::Release => {
                "Release profile (--release) with optimizations enabled".to_string()
            }
            Profile::Test => "Test profile for running tests with appropriate settings".to_string(),
            Profile::Bench => {
                "Benchmark profile for performance testing with optimizations".to_string()
            }
            Profile::Doc => "Documentation profile for building documentation".to_string(),
            Profile::Custom(name) => {
                format!(
                    "Custom profile (--profile {}) with user-defined settings",
                    name
                )
            }
        }
    }

    pub fn cargo_args(&self) -> Vec<String> {
        match self.get_name() {
            Some(p) => vec!["--profile".to_string(), p.to_string()],
            None => Vec::new(),
        }
    }

    pub fn is_standard(&self) -> bool {
        match self {
            Profile::Dev | Profile::Release | Profile::Test | Profile::Bench | Profile::Doc => true,
            Profile::None | Profile::Custom(_) => false,
        }
    }

    pub fn standards_profiles() -> Vec<Profile> {
        vec![
            Profile::None,
            Profile::Dev,
            Profile::Release,
            Profile::Test,
            Profile::Bench,
            Profile::Doc,
        ]
    }
}

impl From<&str> for Profile {
    fn from(s: &str) -> Self {
        match s {
            "dev" => Profile::Dev,
            "release" => Profile::Release,
            "test" => Profile::Test,
            "bench" => Profile::Bench,
            "doc" => Profile::Doc,
            custom => Profile::Custom(custom.to_string()),
        }
    }
}

impl From<String> for Profile {
    fn from(s: String) -> Self {
        Profile::from(s.as_str())
    }
}
