use anyhow::Result;
use core::{Config, CoreService};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Simple server example");

    let config = Config {
        name: "simple-example".to_string(),
        version: "0.1.0".to_string(),
    };

    let service = CoreService::new(config);

    println!("Processing example data...");
    let result = service.process_data("example input")?;
    println!("{}", result);

    // Simulate some work
    sleep(Duration::from_millis(50)).await;

    println!("Simple server example completed");

    Ok(())
}
