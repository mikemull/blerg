CREATE TABLE packets (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    captured_at TIMESTAMPTZ NOT NULL,
    source_mac  MACADDR NOT NULL,
    dest_mac    MACADDR NOT NULL,
    ethertype   INTEGER NOT NULL,   -- 0-65535, doesn't fit in SMALLINT
    payload     BYTEA NOT NULL
);

CREATE INDEX packets_captured_at_idx ON packets USING BRIN (captured_at);
CREATE INDEX packets_source_mac_idx  ON packets (source_mac);

-- Least-privilege role: only what's needed to write into packets.
CREATE ROLE packet_writer WITH LOGIN PASSWORD 'change_me' NOSUPERUSER NOCREATEDB NOCREATEROLE;

GRANT CONNECT ON DATABASE your_database TO packet_writer;
GRANT USAGE ON SCHEMA public TO packet_writer;
GRANT INSERT ON packets TO packet_writer;

-- IEEE OUI (vendor prefix) registry, refreshed via scripts/refresh_oui_vendors.sh.
-- `oui` is the lowercase, colon-separated first 3 octets, e.g. '00:50:c2' --
-- matching the format MacAddr's Display impl produces, so lookups are a
-- plain substring match against source_mac::text / dest_mac::text.
CREATE TABLE oui_vendors (
    oui    TEXT PRIMARY KEY,
    vendor TEXT NOT NULL
);

GRANT SELECT ON oui_vendors TO packet_writer;
