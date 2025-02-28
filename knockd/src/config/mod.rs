use std::fs::File;
use std::io::{Error, Read};

pub use config::Config;
pub mod config;

pub fn load_config(path: &str) -> Result<Config, Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;
    let config: Config = serde_json::from_str(&content)?;

    Ok(config)
}

// test case for load_config
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = load_config("config.json").unwrap();
        assert_eq!(config.interface, "eth0");
        assert_eq!(config.rules.len(), 2);
    }
}
