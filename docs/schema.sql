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
