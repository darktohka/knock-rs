use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub sequence: Vec<u16>,
    pub activate: String,
    pub deactivate: Option<String>,
    pub setup: Option<String>,
    pub teardown: Option<String>,
    pub timeout: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub interface: String,
    pub timeout: u128,
    pub rules: Vec<Rule>,
}
