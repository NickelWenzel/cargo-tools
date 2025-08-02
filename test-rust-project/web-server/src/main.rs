use anyhow::Result;
use core::{Config, CoreService};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting web server...");

    let config = Config::default();
    let service = CoreService::new(config);

    // Simulate server initialization
    println!(
        "Initializing server with config: {:?}",
        service.get_config()
    );

    // Simulate some async work
    for i in 1..=3 {
        println!("Server tick {}", i);
        let data = format!("server-data-{}", i);
        let processed = service.process_data(&data)?;
        println!("{}", processed);

        sleep(Duration::from_millis(100)).await;
    }

    println!("Web server running (simulated)");
    println!("Press Ctrl+C to stop");

    // In a real server, this would be the main event loop
    sleep(Duration::from_secs(1)).await;

    Ok(())
}
