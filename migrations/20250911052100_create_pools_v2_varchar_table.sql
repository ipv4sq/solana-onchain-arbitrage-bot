-- Create pools_v2 table with VARCHAR for Solana addresses
CREATE TABLE IF NOT EXISTS pools_v2 (
    address VARCHAR(44) PRIMARY KEY,
    name VARCHAR NOT NULL,
    dex_type VARCHAR NOT NULL,
    base_mint VARCHAR(44) NOT NULL,
    quote_mint VARCHAR(44) NOT NULL,
    description JSONB NOT NULL,
    data_snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create B-tree index on base_mint (efficient for equality and range queries)
CREATE INDEX idx_pools_v2_base_mint ON pools_v2 USING btree(base_mint);

-- Create B-tree index on quote_mint
CREATE INDEX idx_pools_v2_quote_mint ON pools_v2 USING btree(quote_mint);

-- Create composite index for order-independent mint pair lookups
CREATE INDEX idx_pools_v2_base_quote ON pools_v2 USING btree(base_mint, quote_mint);
CREATE INDEX idx_pools_v2_quote_base ON pools_v2 USING btree(quote_mint, base_mint);

-- Create index on dex_type for filtering by DEX
CREATE INDEX idx_pools_v2_dex_type ON pools_v2(dex_type);

-- Create index on created_at for time-based queries
CREATE INDEX idx_pools_v2_created_at ON pools_v2(created_at);

-- Create trigger to update updated_at on row update
CREATE TRIGGER update_pools_v2_updated_at BEFORE UPDATE ON pools_v2
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();