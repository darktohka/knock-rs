use crate::config::Config;
use crate::config::Rule;
use log::info;
use std::collections::HashMap;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub fn execute_sequence(host: String, sequence: &Vec<u16>, quiet: bool) -> Result<(), Error> {
    // Iterate over the ports and attempt to connect to each
    for port in sequence.iter() {
        let address = format!("{}:{}", host, port);
        let addr: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if !quiet {
            info!("Knocking at: {}", addr[0]);
        }

        // Attempt to connect to the target IP and port
        if let Ok(stream) = TcpStream::connect_timeout(&addr[0], Duration::from_millis(100)) {
            drop(stream);
        }
    }

    if !quiet {
        info!("Rule execution complete.");
    }

    Ok(())
}

pub struct RuleExecutor {
    rules: HashMap<String, Rule>,
    quiet: bool,
}

impl RuleExecutor {
    #[must_use]
    pub fn new(config: Config, quiet: bool) -> RuleExecutor {
        let mut rules = HashMap::new();
        for rule in config.rules {
            rules.insert(rule.name.clone(), rule);
        }

        RuleExecutor { rules, quiet }
    }

    pub fn run(&self, name: &str, host: Option<String>) -> Result<(), Error> {
        if let Some(rule) = self.rules.get(name) {
            info!("Executing rule: {}", rule.name);

            let actual_host = match host {
                Some(host) => host,
                None => match &rule.host {
                    Some(rule_host) => rule_host.to_string(),
                    None => {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "No host provided for rule.",
                        ))
                    }
                },
            };

            return execute_sequence(actual_host, &rule.sequence, self.quiet);
        }

        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            format!("Rule not found: {}", name),
        ));
    }
}
