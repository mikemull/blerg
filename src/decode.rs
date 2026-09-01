use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

// IEEE reserves all values <= 1500 (0x05DC) so this field can never be
// confused with a real EtherType: in classic 802.3 framing it's a length
// field instead, and the frame that follows is LLC (802.2), not IP/ARP/etc.
const IEEE_8023_MAX_LENGTH: u16 = 0x05DC;

// Best-effort decode of an Ethernet payload (everything after the 14-byte
// Ethernet header) into a human-readable summary. Stops at the transport
// layer; doesn't walk IPv6 extension headers or parse LLC framing.
pub fn decode_summary(ethertype: u16, payload: &[u8]) -> String {
    if ethertype <= IEEE_8023_MAX_LENGTH {
        format!("802.3 length={} (LLC framing, not parsed)", ethertype)
    } else if ethertype == EtherTypes::Ipv4.0 {
        match Ipv4Packet::new(payload) {
            Some(ip) => describe_ip(
                ip.get_source().to_string(),
                ip.get_destination().to_string(),
                ip.get_next_level_protocol(),
                ip.payload(),
            ),
            None => "IPv4 (truncated)".to_string(),
        }
    } else if ethertype == EtherTypes::Ipv6.0 {
        match Ipv6Packet::new(payload) {
            Some(ip) => describe_ip(
                ip.get_source().to_string(),
                ip.get_destination().to_string(),
                ip.get_next_header(),
                ip.payload(),
            ),
            None => "IPv6 (truncated)".to_string(),
        }
    } else {
        format!("non-IP ethertype=0x{:04x}", ethertype)
    }
}

fn describe_ip(
    src: String,
    dst: String,
    protocol: IpNextHeaderProtocol,
    transport_payload: &[u8],
) -> String {
    match protocol {
        IpNextHeaderProtocols::Tcp => match TcpPacket::new(transport_payload) {
            Some(tcp) => format!(
                "TCP {}:{} -> {}:{}",
                src, tcp.get_source(), dst, tcp.get_destination()
            ),
            None => format!("TCP {} -> {} (truncated)", src, dst),
        },
        IpNextHeaderProtocols::Udp => match UdpPacket::new(transport_payload) {
            Some(udp) => format!(
                "UDP {}:{} -> {}:{}",
                src, udp.get_source(), dst, udp.get_destination()
            ),
            None => format!("UDP {} -> {} (truncated)", src, dst),
        },
        other => format!("{} {} -> {}", other, src, dst),
    }
}
