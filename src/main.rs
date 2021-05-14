
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}, util::MacAddr};

use std::env;
use std::collections::HashMap;

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

    stats::count_packets(&interface, npacket, mac_map);
}
