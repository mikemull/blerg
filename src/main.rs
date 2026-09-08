
extern crate pnet;

use pnet::{datalink::{self, NetworkInterface}, util::MacAddr};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

use postgres::types::ToSql;
use postgres::{Client, NoTls};

use decode::decode_summary;
use sink::{PacketSink, PgMacAddr, PostgresSink, StdoutSink};

mod decode;
mod macfile;
mod sink;
mod stats;

fn main() {
    let matches = Command::new("blerg")
        .version("0.1")
        .author("Mike <mike.mull@gmail.com>")
        .subcommand_required(true)
        .subcommand(
            Command::new("capture")
                .about("Capture packets from a live interface")
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
        )
        .subcommand(
            Command::new("decode")
                .about("Decode packets already stored in Postgres")
                .arg(Arg::new("db-url")
                    .help("Postgres connection string to read packets from")
                    .long("db-url")
                    .value_name("URL")
                    .required(true))
                .arg(Arg::new("limit")
                    .value_parser(value_parser!(i64))
                    .help("Maximum number of packets to decode, most recent first")
                    .long("limit")
                    .default_value("100"))
                .arg(Arg::new("mac")
                    .help("Only decode packets involving this MAC address as source or destination (repeatable)")
                    .long("mac")
                    .value_name("MAC")
                    .action(ArgAction::Append))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("capture", sub_matches)) => run_capture(sub_matches),
        Some(("decode", sub_matches)) => run_decode(sub_matches),
        _ => unreachable!("subcommand_required guarantees one of the above"),
    }
}

fn run_capture(matches: &ArgMatches) {
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

fn run_decode(matches: &ArgMatches) {
    let db_url = matches.get_one::<String>("db-url").unwrap();
    let limit: i64 = *matches.get_one::<i64>("limit").unwrap();
    let macs: Vec<MacAddr> = matches
        .get_many::<String>("mac")
        .unwrap_or_default()
        .map(|s| MacAddr::from_str(s).unwrap_or_else(|e| panic!("Invalid MAC address '{}': {}", s, e)))
        .collect();

    let mut client = Client::connect(db_url, NoTls).expect("Failed to connect to Postgres");

    // Match if either the source or destination MAC is one of the filter
    // addresses; an empty filter means "everything".
    let mac_clauses: Vec<String> = (0..macs.len())
        .map(|i| format!("(p.source_mac = ${} OR p.dest_mac = ${})", i + 1, i + 1))
        .collect();
    let where_clause = if mac_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", mac_clauses.join(" OR "))
    };

    let mac_params: Vec<PgMacAddr> = macs.into_iter().map(PgMacAddr).collect();
    let mut params: Vec<&(dyn ToSql + Sync)> = mac_params.iter().map(|m| m as &(dyn ToSql + Sync)).collect();
    params.push(&limit);

    // oui_vendors.oui is the lowercase first-3-octets prefix (see
    // docs/schema.sql), so the join matches it against a substring of the
    // full MAC's text form rather than an equality on the MAC itself.
    let query = format!(
        "SELECT p.captured_at, p.source_mac::text, sv.vendor, p.dest_mac::text, dv.vendor, p.ethertype, p.payload \
         FROM packets p \
         LEFT JOIN oui_vendors sv ON sv.oui = substring(p.source_mac::text from 1 for 8) \
         LEFT JOIN oui_vendors dv ON dv.oui = substring(p.dest_mac::text from 1 for 8) \
         {} ORDER BY p.captured_at DESC LIMIT ${}",
        where_clause,
        params.len()
    );

    let rows = client.query(&query, &params).expect("Query failed");

    for row in rows {
        let captured_at: std::time::SystemTime = row.get(0);
        let source_mac: String = row.get(1);
        let source_vendor: Option<String> = row.get(2);
        let dest_mac: String = row.get(3);
        let dest_vendor: Option<String> = row.get(4);
        let ethertype: i32 = row.get(5);
        let payload: Vec<u8> = row.get(6);

        let summary = decode_summary(ethertype as u16, &payload);
        println!(
            "{} {} -> {}  {}",
            humantime::format_rfc3339_seconds(captured_at),
            with_vendor(&source_mac, source_vendor),
            with_vendor(&dest_mac, dest_vendor),
            summary
        );
    }
}

fn with_vendor(mac: &str, vendor: Option<String>) -> String {
    match vendor {
        Some(v) => format!("{} ({})", mac, v),
        None => mac.to_string(),
    }
}
