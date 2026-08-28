use std::error::Error;
use std::time::SystemTime;

use bytes::BytesMut;
use pnet::util::MacAddr;
use postgres::{Client, NoTls};
use postgres_types::{to_sql_checked, IsNull, ToSql, Type};

// Wraps `MacAddr` so we can give it a `ToSql` impl for Postgres's native
// `macaddr` type — the on-wire binary format is just the 6 address bytes.
#[derive(Debug)]
struct PgMacAddr(MacAddr);

impl ToSql for PgMacAddr {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let MacAddr(a, b, c, d, e, f) = self.0;
        out.extend_from_slice(&[a, b, c, d, e, f]);
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::MACADDR
    }

    to_sql_checked!();
}

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

// Writes each record straight into a `packets` table (see ../docs/schema.sql
// for the DDL) using a plain synchronous connection.
pub struct PostgresSink {
    client: Client,
}

impl PostgresSink {
    pub fn new(conn_str: &str) -> Result<Self, postgres::Error> {
        let client = Client::connect(conn_str, NoTls)?;
        Ok(PostgresSink { client })
    }
}

impl PacketSink for PostgresSink {
    fn write(&mut self, record: &PacketRecord) -> Result<(), Box<dyn Error>> {
        self.client.execute(
            "INSERT INTO packets (captured_at, source_mac, dest_mac, ethertype, payload) \
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &record.timestamp,
                &PgMacAddr(record.source),
                &PgMacAddr(record.destination),
                &(record.ethertype as i32),
                &record.payload,
            ],
        )?;
        Ok(())
    }
}
