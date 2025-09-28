use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Core configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "cargo-tools-test".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

/// Core service for shared functionality
pub struct CoreService {
    config: Config,
}

impl CoreService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn process_data(&self, data: &str) -> Result<String> {
        Ok(format!("[{}] Processed: {}", self.config.name, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_service() {
        let config = Config::default();
        let service = CoreService::new(config);

        let result = service.process_data("test input").unwrap();
        assert!(result.contains("cargo-tools-test"));
        assert!(result.contains("test input"));
    }
}
