use anyhow::Result;
use core::Config;

/// Utility functions for string manipulation
pub mod strings {
    use anyhow::Result;

    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    pub fn word_count(s: &str) -> usize {
        s.split_whitespace().count()
    }

    pub fn validate_email(email: &str) -> Result<bool> {
        Ok(email.contains('@') && email.contains('.'))
    }
}

/// Utility functions for data manipulation
pub mod data {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DataPoint {
        pub id: u32,
        pub value: f64,
        pub label: String,
    }

    impl DataPoint {
        pub fn new(id: u32, value: f64, label: String) -> Self {
            Self { id, value, label }
        }
    }

    pub fn process_data_points(points: &[DataPoint]) -> Result<f64> {
        if points.is_empty() {
            return Ok(0.0);
        }

        let sum: f64 = points.iter().map(|p| p.value).sum();
        Ok(sum / points.len() as f64)
    }

    pub fn filter_by_threshold(points: &[DataPoint], threshold: f64) -> Vec<DataPoint> {
        points
            .iter()
            .filter(|p| p.value >= threshold)
            .cloned()
            .collect()
    }
}

/// Configuration utilities
pub fn create_default_config() -> Config {
    Config::default()
}

pub fn validate_config(config: &Config) -> Result<()> {
    if config.name.is_empty() {
        anyhow::bail!("Config name cannot be empty");
    }
    if config.version.is_empty() {
        anyhow::bail!("Config version cannot be empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_utilities() {
        assert_eq!(strings::capitalize("hello"), "Hello");
        assert_eq!(strings::reverse("hello"), "olleh");
        assert_eq!(strings::word_count("hello world test"), 3);
        assert!(strings::validate_email("test@example.com").unwrap());
        assert!(!strings::validate_email("invalid-email").unwrap());
    }

    #[test]
    fn test_data_utilities() {
        use data::*;

        let points = vec![
            DataPoint::new(1, 10.0, "A".to_string()),
            DataPoint::new(2, 20.0, "B".to_string()),
            DataPoint::new(3, 30.0, "C".to_string()),
        ];

        let avg = process_data_points(&points).unwrap();
        assert_eq!(avg, 20.0);

        let filtered = filter_by_threshold(&points, 15.0);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_config_utilities() {
        let config = create_default_config();
        assert!(validate_config(&config).is_ok());
    }
}
