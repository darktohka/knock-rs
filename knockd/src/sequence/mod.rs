pub use port_sequence::PortSequenceDetector;

mod port_sequence;

pub trait SequenceDetector {
    fn start(&mut self);
    fn add_sequence(&mut self, client_ip: String, sequence: u16);
    fn match_sequence(&mut self, client_ip: &str) -> bool;
}
