use anyhow::Result;
use single_crate_core::{Config, CoreService};

fn main() -> Result<()> {
    println!("Single Crate Example: Data Processing Demo");

    // Use default configuration
    let config = Config::default();
    let service = CoreService::new(config);

    println!("Using default configuration: {:?}", service.get_config());

    // Simulate processing different types of data
    let data_samples = vec![
        ("JSON", r#"{"key": "value", "number": 42}"#),
        ("CSV", "name,age,city\nJohn,30,NYC"),
        ("Plain Text", "This is a simple text message"),
        ("Log Entry", "[2023-09-28T10:30:00] INFO: System started"),
    ];

    println!("\nProcessing different data formats:");
    for (format, data) in data_samples {
        println!("\n--- Processing {} ---", format);
        let result = service.process_data(data)?;
        println!("{}", result);
    }

    println!("\nData processing example completed!");

    Ok(())
}