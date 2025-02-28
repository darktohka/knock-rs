use log::{error, info};

use crate::{config::Config, executor};

pub fn ensure_setup(config: &Config) {
    info!("Running setup commands...");

    for rule in &config.rules {
        if let Some(setup) = &rule.setup {
            match executor::execute_command(&setup) {
                Ok(_) => (),
                Err(e) => error!("Failed to execute setup command: {}", e),
            }
        }
    }
}

pub fn ensure_teardown(config: Config) {
    info!("Running teardown commands...");

    for rule in &config.rules {
        if let Some(teardown) = &rule.teardown {
            match executor::execute_command(&teardown) {
                Ok(_) => (),
                Err(e) => error!("Failed to execute teardown command: {}", e),
            }
        }
    }
}
