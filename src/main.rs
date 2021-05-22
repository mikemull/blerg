
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}};
use clap::{Arg, App};

use std::env;

mod macfile;
mod stats;

fn main() {
    let matches = App::new("blerg")
    .version("0.1")
    .author("Mike <mike.mull@gmail.com>")
    .about("Counts packets")
    .arg(Arg::new("INTERFACE")
        .about("Interface to use")
        .required(true)
        .index(1))        
    .arg(Arg::new("NUMPACKETS")
        .about("Stop after this many packets")
        .required(true)
        .index(2))
    .arg(Arg::new("unknown")
        .about("Only list addresses not in MACs file")
        .short('u')
        .long("unknown"))
    .get_matches();

    let interface_name = matches.value_of("INTERFACE").unwrap();
    let npacket: i32 = matches.value_of_t("NUMPACKETS").unwrap();
    let only_unknown: bool = matches.is_present("unknown");

    let mac_map = match macfile::read_mac_file() {
        Ok(mac_map) => mac_map,
        Err(e) => panic!("No MAC mapping file: {}", e)
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
        if only_unknown {
            if !mac_map.contains_key(address) {
                println!("{}: {}", address, count);
            }
        } else {
            println!("{}({}): {}", mac_map.get(address).unwrap_or(&"".to_string()), address, count);
        }
    }
}
