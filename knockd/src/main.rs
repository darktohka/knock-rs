use std::io::Error;

use sequence::PortSequenceDetector;
use server::Server;

mod config;
mod executor;
mod sequence;
mod server;
use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "A port knocking server written in Rust")]
struct Args {
    #[argh(
        option,
        short = 'c',
        default = "String::from(\"config.json\")",
        description = "path to the configuration file"
    )]
    config: String,
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    // Load the configuration
    let config = config::load_config(&args.config)?;
    // Create the sequence detector
    let detector = PortSequenceDetector::new(config.clone());

    let mut server = Server::new(config.interface, Box::new(detector));
    server.start();

    Ok(())
}
