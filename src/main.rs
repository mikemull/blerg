
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}};
use clap::{Arg, ArgAction, Command, value_parser};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

use sink::{PacketSink, PostgresSink, StdoutSink};

mod macfile;
mod sink;
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
        .help("Stop after this many packets (omit to run until stopped with Ctrl-C)")
        .required(false)
        .index(2))
    .arg(Arg::new("unknown")
        .help("Only list addresses not in MACs file")
        .short('u')
        .long("unknown")
        .action(ArgAction::SetTrue))
    .arg(Arg::new("stream")
        .help("Stream each packet to a sink instead of printing a MAC count summary")
        .long("stream")
        .action(ArgAction::SetTrue))
    .arg(Arg::new("db-url")
        .help("Postgres connection string to stream packets into (implies --stream)")
        .long("db-url")
        .value_name("URL"))
    .get_matches();

    let interface_name = matches.get_one::<String>("INTERFACE").unwrap();
    let npacket: Option<i32> = matches.get_one::<i32>("NUMPACKETS").copied();
    let only_unknown: bool = matches.get_flag("unknown");
    let db_url = matches.get_one::<String>("db-url");
    let stream_mode: bool = matches.get_flag("stream") || db_url.is_some();

    let running = Arc::new(AtomicBool::new(true));
    let running_handler = running.clone();
    ctrlc::set_handler(move || running_handler.store(false, Ordering::Relaxed))
        .expect("Error setting Ctrl-C handler");

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

    if stream_mode {
        // Capture stays on this thread; a separate thread owns the sink so a
        // slow write (e.g. to a database) never blocks packet capture.
        let (tx, rx) = mpsc::channel();
        let mut sink: Box<dyn PacketSink> = match db_url {
            Some(url) => Box::new(
                PostgresSink::new(url).expect("Failed to connect to Postgres")
            ),
            None => Box::new(StdoutSink),
        };

        let writer = thread::spawn(move || {
            for record in rx {
                if let Err(e) = sink.write(&record) {
                    eprintln!("sink error: {}", e);
                }
            }
        });

        stats::stream_packets(&interface, npacket, running, tx);
        writer.join().unwrap();
    } else {
        // Count packets by MAC address
        let packet_counts = stats::count_packets(&interface, npacket, running);

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
}
