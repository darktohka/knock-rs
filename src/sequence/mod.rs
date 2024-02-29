mod port_sequence;

pub trait SequenceDetector {
    fn add_sequence(&mut self, client_ip: String, sequence: i32);
    fn match_sequence(&self, client_ip: &str) -> bool;
}
