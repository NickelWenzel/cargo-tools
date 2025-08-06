use anyhow::Result;
use clap::{Arg, Command};
use core::{Config, CoreService};

fn main() -> Result<()> {
    let matches = Command::new("cli-tool")
        .version("0.1.0")
        .about("CLI tool binary")
        .arg(
            Arg::new("action")
                .help("Action to perform")
                .required(true)
                .value_parser(["check", "validate", "format"])
                .index(1),
        )
        .get_matches();

    let config = Config::default();
    let service = CoreService::new(config);

    let action = matches.get_one::<String>("action").unwrap();

    match action.as_str() {
        "check" => {
            let result = service.process_data("checking...")?;
            println!("{result}");
        }
        "validate" => {
            let result = service.process_data("validating...")?;
            println!("{result}");
        }
        "format" => {
            let result = service.process_data("formatting...")?;
            println!("{result}");
        }
        _ => unreachable!(),
    }

    Ok(())
}
