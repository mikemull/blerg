
use pnet::{datalink::{self, NetworkInterface}, util::MacAddr};
use pnet::packet::ethernet::EthernetPacket;
use pnet::datalink::Channel::Ethernet;
use std::collections::HashMap;


pub fn count_packets(interface: &NetworkInterface, num_packets: i32, mac_map: HashMap<MacAddr, String>) {

    let mut packet_counts = HashMap::new();

    // Create a new channel, dealing with layer 2 packets
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    let mut pcount = 0;

    while pcount < num_packets {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                //println!("{}", packet.get_source());
                *packet_counts.entry(packet.get_source()).or_insert(0) += 1;

            },
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
        pcount += 1;
    }
    for (address, count) in &packet_counts {
        println!("{}({}): {}", mac_map.get(address).unwrap_or(&"".to_string()), address, count);
    }

}