use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct PacketInfo {
    pub source_ip: String,
    pub destination_port: u16,
}

pub fn parse_ethernet_ip_packet(packet: &[u8]) -> Option<PacketInfo> {
    if packet.len() < 14 + 20 + 4 {
        // Ethernet header is 14 bytes, at least 20 bytes for IPv4 header, 4 bytes for TCP header
        return None;
    }

    // Check if EtherType indicates IPv4 or IPv6 (bytes 12-13 in Ethernet header)
    match (packet[12], packet[13]) {
        (0x08, 0x00) => parse_ipv4_packet(&packet[14..]),
        (0x86, 0xDD) => parse_ipv6_packet(&packet[14..]),
        _ => None,
    }
}

fn parse_ipv4_packet(packet: &[u8]) -> Option<PacketInfo> {
    if packet.len() < 20 + 4 {
        // Minimum IPv4 header is 20 bytes
        return None;
    }

    // Extract source IP (bytes 12-15 in IPv4 header)
    let source_ip = Ipv4Addr::new(packet[12], packet[13], packet[14], packet[15]).to_string();

    // IHL field in the first byte (lower nibble) specifies IPv4 header length in 32-bit words
    let ihl = (packet[0] & 0x0F) as usize * 4;

    if ihl < 20 || packet.len() < ihl + 4 {
        // IPv4 header must be at least 20 bytes
        return None;
    }

    // Extract destination port from TCP header (bytes 2-3 after IPv4 header)
    let destination_port = u16::from_be_bytes([packet[ihl + 2], packet[ihl + 3]]);

    Some(PacketInfo {
        source_ip,
        destination_port,
    })
}

fn parse_ipv6_packet(packet: &[u8]) -> Option<PacketInfo> {
    if packet.len() < 40 + 4 {
        // Minimum IPv6 header is 40 bytes, 4 bytes for TCP header
        return None;
    }

    // Extract source IP (bytes 8-23 in IPv6 header)
    let source_ip = format!(
        "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
        u16::from_be_bytes([packet[8], packet[9]]),
        u16::from_be_bytes([packet[10], packet[11]]),
        u16::from_be_bytes([packet[12], packet[13]]),
        u16::from_be_bytes([packet[14], packet[15]]),
        u16::from_be_bytes([packet[16], packet[17]]),
        u16::from_be_bytes([packet[18], packet[19]]),
        u16::from_be_bytes([packet[20], packet[21]]),
        u16::from_be_bytes([packet[22], packet[23]]),
    );

    // Extract destination port from TCP header (bytes 2-3 after IPv6 header)
    let destination_port = u16::from_be_bytes([packet[40 + 2], packet[40 + 3]]);

    Some(PacketInfo {
        source_ip,
        destination_port,
    })
}
