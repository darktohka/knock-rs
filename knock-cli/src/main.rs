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
    rule: Option<String>,
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    let rule = match args.rule {
        Some(rule) => rule,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "No rule specified.",
            ));
        }
    };

    let config = config::load_config(&args.config)?;
    let executor = rule::RuleExecutor::new(config);

    executor.run(&rule)
}
