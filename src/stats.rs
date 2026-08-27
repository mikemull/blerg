
use pnet::{datalink::{self, NetworkInterface}, util::MacAddr};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::Packet;
use pnet::datalink::Channel::Ethernet;
use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::sink::PacketRecord;

// How often a blocked read wakes up to check whether it should stop, when
// there's no packet traffic to wake it naturally.
const READ_TIMEOUT: Duration = Duration::from_millis(200);

fn open_channel(interface: &NetworkInterface) -> Box<dyn datalink::DataLinkReceiver> {
    let config = datalink::Config {
        read_timeout: Some(READ_TIMEOUT),
        ..Default::default()
    };

    match datalink::channel(interface, config) {
        Ok(Ethernet(_tx, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    }
}

// Collect Layer 2 packets and keep track of counts by MAC address. Stops
// after num_packets (if given) or when `running` is cleared, whichever
// comes first.
pub fn count_packets(
    interface: &NetworkInterface,
    num_packets: Option<i32>,
    running: Arc<AtomicBool>)
    -> HashMap<MacAddr, i32> {

    let mut packet_counts = HashMap::new();
    let mut rx = open_channel(interface);
    let mut pcount = 0;

    while running.load(Ordering::Relaxed) && num_packets.map_or(true, |n| pcount < n) {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                *packet_counts.entry(packet.get_source()).or_insert(0) += 1;
                pcount += 1;
            },
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }

    return packet_counts;
}

// Collect Layer 2 packets and send an owned record for each one to `tx`,
// for a consumer (e.g. a database writer) to handle. Stops after
// num_packets (if given) or when `running` is cleared, whichever comes
// first.
pub fn stream_packets(
    interface: &NetworkInterface,
    num_packets: Option<i32>,
    running: Arc<AtomicBool>,
    tx: Sender<PacketRecord>) {

    let mut rx = open_channel(interface);
    let mut pcount = 0;

    while running.load(Ordering::Relaxed) && num_packets.map_or(true, |n| pcount < n) {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                let record = PacketRecord {
                    timestamp: SystemTime::now(),
                    source: packet.get_source(),
                    destination: packet.get_destination(),
                    ethertype: packet.get_ethertype().0,
                    payload: packet.payload().to_vec(),
                };
                // Consumer thread has gone away; nothing left to do.
                if tx.send(record).is_err() {
                    break;
                }
                pcount += 1;
            },
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}
