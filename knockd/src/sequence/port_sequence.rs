use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::executor;
use crate::sequence::SequenceDetector;
use log::{error, info};

// Default rule timeout is 6 hours
pub const DEFAULT_RULE_TIMEOUT: u128 = 6 * 60 * 60;

#[derive(Debug, Clone)]
pub struct RuleCommands {
    activate: String,
    deactivate: Option<String>,
    timeout: u128,
}

#[derive(Debug)]
pub struct PortSequenceDetector {
    timeout: u128,
    sequence_set: HashSet<u16>,
    sequence_rules: HashMap<String, Vec<u16>>,
    rules_map: HashMap<String, RuleCommands>,
    client_sequences: Arc<Mutex<HashMap<String, Vec<u16>>>>,
    client_timeout: Arc<Mutex<HashMap<String, u128>>>,
    active_rules: Arc<Mutex<HashMap<String, HashMap<String, u128>>>>,
    update_signal: Arc<(Mutex<bool>, std::sync::Condvar)>,
}

impl PortSequenceDetector {
    #[must_use]
    pub fn new(config: Config) -> PortSequenceDetector {
        let mut sequence_rules = HashMap::new();
        for rule in config.rules.clone() {
            sequence_rules.insert(rule.name, rule.sequence);
        }

        let mut sequence_set = HashSet::new();
        for rule in config.rules.clone() {
            for sequence in rule.sequence {
                sequence_set.insert(sequence);
            }
        }

        let mut rules_map = HashMap::new();
        for rule in config.rules {
            rules_map.insert(
                rule.name,
                RuleCommands {
                    activate: rule.activate,
                    deactivate: rule.deactivate,
                    timeout: rule.timeout.unwrap_or(DEFAULT_RULE_TIMEOUT),
                },
            );
        }

        PortSequenceDetector {
            timeout: config.timeout,
            sequence_set,
            sequence_rules,
            rules_map,
            client_sequences: Arc::new(Mutex::new(HashMap::new())),
            client_timeout: Arc::new(Mutex::new(HashMap::new())),
            active_rules: Arc::new(Mutex::new(HashMap::new())),
            update_signal: Arc::new((Mutex::new(false), std::sync::Condvar::new())),
        }
    }
}

impl SequenceDetector for PortSequenceDetector {
    fn add_sequence(&mut self, client_ip: String, sequence: u16) {
        // check if the sequence is in the set
        if !self.sequence_set.contains(&sequence) {
            return;
        }

        info!(
            "SYN packet detected from: {} to target port: {}",
            client_ip, sequence
        );

        {
            let mut client_sequence = self.client_sequences.lock().unwrap();
            let client_sequence = client_sequence
                .entry(client_ip.clone())
                .or_insert(Vec::new());
            client_sequence.push(sequence);

            // get the current time stamp
            let mut client_timeout = self.client_timeout.lock().unwrap();
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            client_timeout.insert(client_ip.clone(), current_time);
        }

        self.match_sequence(&client_ip);

        // Notify the detector thread
        let (lock, cvar) = &*self.update_signal;
        let mut update_signal = lock.lock().unwrap();
        *update_signal = true;
        cvar.notify_all();
    }

    fn match_sequence(&mut self, client_ip: &str) -> bool {
        // Check if the current sequence matches any of the rules
        let mut client_sequence = self.client_sequences.lock().unwrap();
        let client_sequence = client_sequence.get_mut(client_ip);
        if let Some(sequence) = client_sequence {
            for (name, rule) in &self.sequence_rules {
                if sequence.ends_with(&rule) {
                    info!("Matched knock sequence: {:?} from: {}", rule, client_ip);

                    // Clear the sequence
                    sequence.clear();

                    // Remove the timeout
                    self.client_timeout.lock().unwrap().remove(client_ip);

                    // execute the command
                    let command = self.rules_map.get(name).unwrap();
                    let formatted_cmd = command.activate.replace("%IP%", client_ip);
                    info!("Executing activation command: {}", formatted_cmd);

                    // format the command with the client ip
                    match executor::execute_command(&formatted_cmd) {
                        Ok(_) => {
                            info!("Activation command executed successfully");
                        }
                        Err(e) => {
                            error!("Error executing activation command: {:?}", e);
                        }
                    }

                    // Insert the rule for deactivation
                    if let Some(_deactivate) = &command.deactivate {
                        let mut active_rules = self.active_rules.lock().unwrap();
                        let rule_map = active_rules
                            .entry(client_ip.to_string())
                            .or_insert(HashMap::new());

                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        rule_map.insert(name.clone(), time);
                    }

                    return true;
                }
            }
        }

        false
    }

    fn start(&mut self) {
        let client_sequences = Arc::clone(&self.client_sequences);
        let client_timeout = Arc::clone(&self.client_timeout);
        let active_rules = Arc::clone(&self.active_rules);
        let rules_map = self.rules_map.clone();
        let update_signal = Arc::clone(&self.update_signal);
        let timeout = self.timeout;

        thread::spawn(move || {
            let (lock, cvar) = &*update_signal;

            loop {
                // Clear trigger flag
                let mut triggered = lock.lock().unwrap();
                *triggered = false;
                cvar.notify_all();

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let mut sleep_for = std::time::Duration::from_millis(u64::MAX);

                {
                    let client_sequences_guard = match client_sequences.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            error!("Error: {:?}", poisoned);
                            continue;
                        }
                    };

                    let client_timeout_guard = match client_timeout.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            error!("Error: {:?}", poisoned);
                            continue;
                        }
                    };

                    let mut client_sequences = client_sequences_guard;
                    let mut client_timeout = client_timeout_guard;

                    let clients_to_remove: Vec<_> = client_timeout
                        .iter()
                        .filter_map(|(client_ip, _)| {
                            let last_time = client_timeout.get(client_ip).unwrap();

                            if now - last_time >= timeout {
                                // This client should be removed
                                return Some(client_ip.clone());
                            }

                            let client_sleep = last_time + timeout - now;

                            if client_sleep < sleep_for.as_millis() {
                                // We should sleep for the shortest time
                                sleep_for = std::time::Duration::from_millis(client_sleep as u64);
                            }

                            None
                        })
                        .collect();

                    for client_ip in clients_to_remove {
                        client_sequences.remove(&client_ip);
                        client_timeout.remove(&client_ip);
                    }
                }

                {
                    let active_rules_guard = match active_rules.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            error!("Error: {:?}", poisoned);
                            continue;
                        }
                    };
                    let mut active_rules = active_rules_guard;

                    let rules_to_deactivate = active_rules
                        .iter()
                        .filter_map(|(client_ip, rules)| {
                            let mut rules_to_deactivate: Vec<String> = Vec::new();

                            for (rule, last_time) in rules {
                                let rule_timeout = rules_map.get(rule).unwrap().timeout;

                                if now - last_time >= rule_timeout {
                                    // This rule should be deactivated
                                    rules_to_deactivate.push(rule.clone());
                                    continue;
                                }

                                let rule_sleep = last_time + rule_timeout - now;

                                if rule_sleep < sleep_for.as_millis() {
                                    // We should sleep for the shortest time
                                    sleep_for = std::time::Duration::from_millis(rule_sleep as u64);
                                }
                            }

                            if rules_to_deactivate.is_empty() {
                                return None;
                            }

                            Some((client_ip.clone(), rules_to_deactivate))
                        })
                        .collect::<Vec<_>>();

                    for (client_ip, rules) in rules_to_deactivate {
                        for rule in rules {
                            if let Some(deactivate) =
                                rules_map.get(&rule).unwrap().deactivate.as_ref()
                            {
                                let formatted_cmd = deactivate.replace("%IP%", &client_ip);
                                info!("Executing deactivation command: {}", formatted_cmd);

                                // format the command with the client ip
                                match executor::execute_command(&formatted_cmd) {
                                    Ok(_) => {
                                        info!("Deactivation command executed successfully");
                                    }
                                    Err(e) => {
                                        error!("Error executing command: {:?}", e);
                                    }
                                }
                            }

                            // Remove the rule from the client IP
                            if let Some(rules) = active_rules.get_mut(&client_ip) {
                                rules.remove(&rule);
                            }
                        }

                        // Remove client IP if no rules are active
                        if let Some(rules) = active_rules.get(&client_ip) {
                            if rules.is_empty() {
                                active_rules.remove(&client_ip);
                            }
                        }
                    }
                }

                // Wait for the next signal or the next timeout
                let _ = cvar.wait_timeout(triggered, sleep_for).unwrap();
            }
        });

        info!("Port sequence detector thread started...");
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    fn create_config() -> Config {
        Config {
            interface: "enp3s0".to_string(),
            timeout: 2,
            rules: vec![
                crate::config::config::Rule {
                    name: "enable ssh".to_string(),
                    sequence: vec![1, 2, 3],
                    activate: "ls -lh".to_string(),
                    deactivate: None,
                    timeout: None,
                    setup: None,
                    teardown: None,
                },
                crate::config::config::Rule {
                    name: "disable ssh".to_string(),
                    sequence: vec![3, 5, 6],
                    activate: "free -g".to_string(),
                    deactivate: None,
                    timeout: None,
                    setup: None,
                    teardown: None,
                },
            ],
        }
    }

    #[test]
    fn test_new() {
        let config = create_config();
        let detector = PortSequenceDetector::new(config);
        assert_eq!(detector.sequence_set.len(), 5);
        assert_eq!(detector.sequence_rules.len(), 2);
        assert_eq!(detector.timeout, 2);
    }

    #[test]
    fn test_add_sequence() {
        let config = create_config();
        let mut detector = PortSequenceDetector::new(config);
        detector.add_sequence("127.0.0.1".to_owned(), 3);
        let client_sequences = detector.client_sequences.lock().unwrap();
        assert_eq!(client_sequences.get("127.0.0.1"), Some(&vec![3]));
    }

    #[test]
    fn test_add_sequence_with_timeout() {
        let config = create_config();
        let mut detector = PortSequenceDetector::new(config);
        detector.start();
        detector.add_sequence("127.0.0.1".to_owned(), 3);
        thread::sleep(Duration::from_secs(4));
        let client_sequences = detector.client_sequences.lock().unwrap();
        assert_eq!(client_sequences.get("127.0.0.1"), None);
    }

    #[test]
    fn test_add_none_existing_sequence() {
        let config = create_config();
        let mut detector = PortSequenceDetector::new(config);
        detector.add_sequence("127.0.0.1".to_owned(), 9);
        let client_sequences = detector.client_sequences.lock().unwrap();
        assert_eq!(client_sequences.get("127.0.0.1"), None);
    }

    #[test]
    fn test_match_sequence() {
        let config = create_config();
        let mut detector = PortSequenceDetector::new(config);
        detector.add_sequence("127.0.0.1".to_owned(), 1);
        detector.add_sequence("127.0.0.1".to_owned(), 3);
        detector.add_sequence("127.0.0.1".to_owned(), 5);
        detector.add_sequence("127.0.0.1".to_owned(), 6);
        assert_eq!(detector.match_sequence("127.0.0.1"), false);
        let client_sequences = detector.client_sequences.lock().unwrap();
        assert_eq!(client_sequences.get("127.0.0.1").unwrap().len(), 0);
    }
}
