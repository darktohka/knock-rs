use std::io::Error;

use sequence::PortSequenceDetector;
use server::Server;

use argh::FromArgs;
use log::error;
use simple_logger;

mod config;
mod executor;
mod lifetime;
mod sequence;
mod server;

#[derive(FromArgs)]
#[argh(description = "A port knocking server written in Rust")]
struct Args {
    #[argh(
        option,
        short = 'c',
        default = "String::from(\"/etc/knockd/config.json\")",
        description = "path to the configuration file"
    )]
    config: String,
}

fn main() -> Result<(), Error> {
    simple_logger::init().expect("Failed to initialize logger");

    let args: Args = argh::from_env();

    // Load the configuration
    let config = config::load_config(&args.config)?;
    // Create the sequence detector
    let detector = PortSequenceDetector::new(config.clone());
    let interface = config.interface.clone();

    lifetime::ensure_setup(&config);

    ctrlc::set_handler(move || {
        lifetime::ensure_teardown(config.clone());
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut server = Server::new(interface, Box::new(detector));

    if let Err(e) = server.start() {
        error!("Error starting server: {e}");
    }

    Ok(())
}
