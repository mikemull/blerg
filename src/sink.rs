use std::error::Error;
use std::time::SystemTime;

use pnet::util::MacAddr;

// An owned, self-contained record of one captured packet, safe to hold
// past the lifetime of the underlying capture buffer (e.g. to queue it
// for a consumer on another thread).
#[derive(Debug, Clone)]
pub struct PacketRecord {
    pub timestamp: SystemTime,
    pub source: MacAddr,
    pub destination: MacAddr,
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

// Destination for captured packets. Implement this for whatever backend
// should ultimately receive them (a database, a message queue, a file).
pub trait PacketSink: Send {
    fn write(&mut self, record: &PacketRecord) -> Result<(), Box<dyn Error>>;
}

// Placeholder sink until a real backend is chosen.
pub struct StdoutSink;

impl PacketSink for StdoutSink {
    fn write(&mut self, record: &PacketRecord) -> Result<(), Box<dyn Error>> {
        println!(
            "{:?} {} -> {} ethertype=0x{:04x} len={}",
            record.timestamp,
            record.source,
            record.destination,
            record.ethertype,
            record.payload.len()
        );
        Ok(())
    }
}
