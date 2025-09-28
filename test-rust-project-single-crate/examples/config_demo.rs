use anyhow::Result;
use single_crate_core::{Config, CoreService};

fn main() -> Result<()> {
    println!("Single Crate Example: Configuration Demo");

    // Create a custom configuration
    let config = Config {
        name: "example-service".to_string(),
        version: "1.0.0".to_string(),
    };

    let service = CoreService::new(config);

    println!("Service initialized with config:");
    println!("  Name: {}", service.get_config().name);
    println!("  Version: {}", service.get_config().version);

    // Demonstrate processing various types of data
    let test_data = vec![
        "user input",
        "configuration data",
        "test payload",
        "example message",
    ];

    println!("\nProcessing example data:");
    for data in test_data {
        let result = service.process_data(data)?;
        println!("  {}", result);
    }

    println!("\nExample completed successfully!");

    Ok(())
}
