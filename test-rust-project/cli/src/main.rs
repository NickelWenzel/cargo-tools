use anyhow::Result;
use clap::{Arg, Command};
use core::{Config, CoreService};

fn main() -> Result<()> {
    let matches = Command::new("cli-main")
        .version("0.1.0")
        .about("Main CLI application")
        .arg(
            Arg::new("input")
                .help("Input data to process")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config = Config::default();
    let service = CoreService::new(config);

    let input = matches.get_one::<String>("input").unwrap();
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Processing input: {input}");
    }

    let result = service.process_data(input)?;
    println!("{result}");

    Ok(())
}
