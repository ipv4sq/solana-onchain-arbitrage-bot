-- Create mints table
CREATE TABLE IF NOT EXISTS mints (
    address BYTEA PRIMARY KEY CHECK (octet_length(address) = 32),
    symbol VARCHAR NOT NULL,
    decimals SMALLINT NOT NULL,
    program BYTEA NOT NULL CHECK (octet_length(program) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on symbol for searching
CREATE INDEX idx_mints_symbol ON mints(symbol);

-- Create index on program for filtering by token program
CREATE INDEX idx_mints_program ON mints(program);

-- Create or replace the update function (idempotent)
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to update updated_at on row update
CREATE TRIGGER update_mints_updated_at BEFORE UPDATE ON mints
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();