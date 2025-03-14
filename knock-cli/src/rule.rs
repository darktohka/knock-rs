use crate::config::Config;
use crate::config::Rule;
use log::info;
use std::collections::HashMap;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct RuleExecutor {
    rules: HashMap<String, Rule>,
}

impl RuleExecutor {
    #[must_use]
    pub fn new(config: Config) -> RuleExecutor {
        let mut rules = HashMap::new();
        for rule in config.rules {
            rules.insert(rule.name.clone(), rule);
        }

        RuleExecutor { rules }
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

            // Iterate over the ports and attempt to connect to each
            for port in rule.sequence.iter() {
                let address = format!("{}:{}", actual_host, port);
                let addr: Vec<SocketAddr> = address.to_socket_addrs()?.collect();
                info!("Knocking at: {}", addr[0]);

                // Attempt to connect to the target IP and port
                if let Ok(stream) = TcpStream::connect_timeout(&addr[0], Duration::from_millis(100))
                {
                    drop(stream);
                }
            }
        } else {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                format!("Rule not found: {}", name),
            ));
        }

        info!("Rule execution complete.");
        Ok(())
    }
}
