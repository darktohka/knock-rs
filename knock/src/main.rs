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
        default = "String::from(\"/etc/knockd/config.json\")",
        description = "path to the configuration file"
    )]
    config: String,
    #[argh(option, short = 'r', description = "the port knocking rule to execute")]
    rule: String,
    #[argh(option, short = 's', description = "the sequence to play")]
    sequence: Option<String>,
    #[argh(option, short = 'h', description = "the host to connect to")]
    host: Option<String>,
    #[argh(
        option,
        default = "false",
        short = 'q',
        description = "suppress output"
    )]
    quiet: bool,
}

fn main() -> Result<(), Error> {
    simple_logger::init().expect("Failed to initialize logger");

    let args: Args = argh::from_env();

    if let Some(sequence) = args.sequence {
        if let Some(host) = args.host {
            let sequence: Vec<u16> = sequence
                .split(',')
                .map(|s| s.parse().expect("Invalid sequence"))
                .collect();
            return rule::execute_sequence(host, &sequence, args.quiet);
        }
    }

    let config = config::load_config(&args.config)?;
    let executor = rule::RuleExecutor::new(config, args.quiet);

    executor.run(&args.rule, args.host)
}
