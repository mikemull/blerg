
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}, util::MacAddr};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};

use std::{borrow::Borrow, env};
use std::collections::HashMap;
use std::str::FromStr;

mod macfile;


fn main() {
    let interface_opt = env::args().nth(1);
    let npacket_opt = env::args().nth(2);

    if let Err(err) = macfile::read_mac_file() {
        println!("error running example: {}", err);
    }

    let interface_name = match interface_opt {
        Some(p) =>  p,
        None => "en0".to_string()
    };
    let npacket = match npacket_opt {
        Some(p) => p.parse().unwrap(),
        None => 1000
    };

    //let interface_name = env::args().nth(1).unwrap();
    println!("{}", interface_name);
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();

    count_packets(&interface, npacket);
}

fn count_packets(interface: &NetworkInterface, num_packets: i32) {
    let mac_map: HashMap<MacAddr, &str> =
    [(MacAddr::from_str("08:9e:08:f9:30:e6").unwrap(), "Node 1"),
    (MacAddr::from_str("ac:bc:32:97:f0:bf").unwrap(), "Mac"),
    (MacAddr::from_str("08:9e:08:f9:30:e8").unwrap(), "Node 2"),
    (MacAddr::from_str("60:01:94:4c:f6:18").unwrap(), "ESP_4CF618"),
    (MacAddr::from_str("5c:cf:7f:53:31:22").unwrap(), "ESP_533122"),
    (MacAddr::from_str("42:a8:1e:fd:1c:93").unwrap(), "Em-IPhone11"),
    (MacAddr::from_str("34:15:13:25:8e:52").unwrap(), "Envoy")
    ]
    .iter().cloned().collect();

    let mut packet_counts = HashMap::new();

    // Create a new channel, dealing with layer 2 packets
    let (tx, mut rx) = match datalink::channel(&interface, Default::default()) {
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

                // Constructs a single packet, the same length as the the one received,
                // using the provided closure. This allows the packet to be constructed
                // directly in the write buffer, without copying. If copying is not a
                // problem, you could also use send_to.
                //
                // The packet is sent once the closure has finished executing.
                /*tx.build_and_send(1, packet.packet().len(),
                    &mut |mut new_packet| {
                        let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                        // Create a clone of the original packet
                        new_packet.clone_from(&packet);

                        // Switch the source and destination
                        new_packet.set_source(packet.get_destination());
                        new_packet.set_destination(packet.get_source());
                });*/
            },
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
        pcount += 1;
    }
    for (address, count) in &packet_counts {
        println!("{}({}): {}", mac_map.get(address).unwrap_or(&""), address, count);
    }

}