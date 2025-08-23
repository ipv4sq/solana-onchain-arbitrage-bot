-- Create pools table
CREATE TABLE IF NOT EXISTS pools (
    address BYTEA PRIMARY KEY CHECK (octet_length(address) = 32),
    name VARCHAR NOT NULL,
    dex_type VARCHAR NOT NULL,
    base_mint BYTEA NOT NULL CHECK (octet_length(base_mint) = 32),
    quote_mint BYTEA NOT NULL CHECK (octet_length(quote_mint) = 32),
    base_vault BYTEA NOT NULL CHECK (octet_length(base_vault) = 32),
    quote_vault BYTEA NOT NULL CHECK (octet_length(quote_vault) = 32),
    description JSONB NOT NULL,
    data_snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create B-tree index on base_mint (efficient for equality and range queries)
CREATE INDEX idx_pools_base_mint ON pools USING btree(base_mint);

-- Create B-tree index on quote_mint
CREATE INDEX idx_pools_quote_mint ON pools USING btree(quote_mint);

-- Create composite index for order-independent mint pair lookups
-- Using both (base_mint, quote_mint) and (quote_mint, base_mint) combinations
CREATE INDEX idx_pools_base_quote ON pools USING btree(base_mint, quote_mint);
CREATE INDEX idx_pools_quote_base ON pools USING btree(quote_mint, base_mint);

-- Create index on dex_type for filtering by DEX
CREATE INDEX idx_pools_dex_type ON pools(dex_type);

-- Create index on created_at for time-based queries
CREATE INDEX idx_pools_created_at ON pools(created_at);

-- Create trigger to update updated_at on row update
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_pools_updated_at BEFORE UPDATE ON pools
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();