use anyhow::Result;
use single_crate_core::{Config, CoreService};

fn main() -> Result<()> {
    println!("Single Crate Binary Example");

    let config = Config {
        name: "single-crate-binary".to_string(),
        version: "0.1.0".to_string(),
    };

    let service = CoreService::new(config);

    println!("Running binary with config: {:?}", service.get_config());

    // Process some example data
    let inputs = vec!["hello", "world", "from", "binary"];

    for input in inputs {
        let result = service.process_data(input)?;
        println!("{}", result);
    }

    println!("Binary execution completed successfully!");

    Ok(())
}
