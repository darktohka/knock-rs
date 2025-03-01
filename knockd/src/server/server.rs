use std::io::Error;

use crate::sequence::SequenceDetector;
use crate::server::parser::parse_ethernet_ip_packet;
use log::warn;

use pcap::{Capture, Device};

pub struct Server {
    interface_name: String,
    detector: Box<dyn SequenceDetector>,
}

impl Server {
    pub fn new(interface: String, detector: Box<dyn SequenceDetector>) -> Box<Server> {
        Box::new(Server {
            interface_name: interface,
            detector,
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        // Start the sequence detector thread
        self.detector.start();

        // Find the network device by name
        let device = Device::list()
            .unwrap()
            .into_iter()
            .find(|d| d.name == self.interface_name)
            .expect("Failed to get interface");

        // Open the capture handle
        let mut cap = Capture::from_device(device)
            .unwrap()
            .promisc(true)
            .snaplen(256)
            .timeout(100)
            .open()
            .unwrap();

        // Apply a BPF filter to capture only SYN packets
        // Right now, BPF filters do not support introspecting IPv6 packets...
        // Hopefully this will be fixed later. I'm putting this here in case they fix it.
        cap.filter(
            "(ip or ip6) and tcp[tcpflags] & (tcp-syn|tcp-ack) == tcp-syn",
            true,
        )
        .unwrap();

        while let Ok(packet) = cap.next_packet() {
            if let Some(info) = parse_ethernet_ip_packet(&packet) {
                self.detector
                    .add_sequence(info.source_ip, info.destination_port);
            }
        }

        warn!("Packet sniffing interrupted.");
        Ok(())
    }
}
