
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}};

use std::env;

mod macfile;
mod stats;

fn main() {
    let interface_opt = env::args().nth(1);
    let npacket_opt = env::args().nth(2);

    let mac_map = macfile::read_mac_file().unwrap();
    //if let Err(err) = macfile::read_mac_file() {
    //    println!("error running example: {}", err);
    //}

    let interface_name = match interface_opt {
        Some(p) =>  p,
        None => "en0".to_string()
    };
    let npacket = match npacket_opt {
        Some(p) => p.parse().unwrap(),
        None => 1000
    };

    println!("{}", interface_name);
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();

    // Count packets by MAC address
    let packet_counts = stats::count_packets(&interface, npacket);

    for (address, count) in &packet_counts {
        println!("{}({}): {}", mac_map.get(address).unwrap_or(&"".to_string()), address, count);
    }
}
