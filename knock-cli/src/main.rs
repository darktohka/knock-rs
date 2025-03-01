use std::io::Error;

mod config;
mod rule;

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "A port knocking console application written in Rust")]
struct Args {
    #[argh(
        option,
        short = 'c',
        default = "String::from(\"config.json\")",
        description = "path to the configuration file"
    )]
    config: String,
    #[argh(option, short = 'r', description = "the port knocking rule to execute")]
    rule: String,
    #[argh(option, short = 'h', description = "the host to connect to")]
    host: Option<String>,
}

fn main() -> Result<(), Error> {
    simple_logger::init().expect("Failed to initialize logger");

    let args: Args = argh::from_env();

    let config = config::load_config(&args.config)?;
    let executor = rule::RuleExecutor::new(config);

    executor.run(&args.rule, args.host)
}
