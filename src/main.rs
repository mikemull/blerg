
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}};
use clap::{Arg, ArgAction, Command, value_parser};

mod macfile;
mod stats;

fn main() {
    let matches = Command::new("blerg")
    .version("0.1")
    .author("Mike <mike.mull@gmail.com>")
    .arg(Arg::new("INTERFACE")
        .help("Interface to use")
        .required(true)
        .index(1))        
    .arg(Arg::new("NUMPACKETS")
        .value_parser(value_parser!(i32))
        .help("Stop after this many packets")
        .required(true)
        .index(2))
    .arg(Arg::new("unknown")
        .help("Only list addresses not in MACs file")
        .short('u')
        .long("unknown")
        .action(ArgAction::SetTrue))
    .get_matches();

    let interface_name = matches.get_one::<String>("INTERFACE").unwrap();
    let npacket: i32 = *matches.get_one::<i32>("NUMPACKETS").unwrap();
    let only_unknown: bool = matches.get_flag("unknown");

    let mac_map = match macfile::read_mac_file() {
        Ok(mac_map) => mac_map,
        Err(e) => panic!("No MAC mapping file: {}", e)
    };

    println!("{}", interface_name);
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == *interface_name;

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
