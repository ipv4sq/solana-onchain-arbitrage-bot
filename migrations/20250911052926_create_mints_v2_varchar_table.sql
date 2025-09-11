-- Create mints_v2 table with VARCHAR for Solana addresses
CREATE TABLE IF NOT EXISTS mints_v2 (
    address VARCHAR(44) PRIMARY KEY,
    symbol VARCHAR NOT NULL,
    decimals SMALLINT NOT NULL,
    program VARCHAR(44) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on symbol for searching
CREATE INDEX idx_mints_v2_symbol ON mints_v2(symbol);

-- Create index on program for filtering by token program
CREATE INDEX idx_mints_v2_program ON mints_v2(program);

-- Create trigger to update updated_at on row update
CREATE TRIGGER update_mints_v2_updated_at BEFORE UPDATE ON mints_v2
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
