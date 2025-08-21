-- Initial database schema migration
-- Created from existing database structure

-- Create pool_mints table
CREATE TABLE IF NOT EXISTS pool_mints (
    id SERIAL PRIMARY KEY,
    pool_id VARCHAR(88) NOT NULL,
    desired_mint VARCHAR(88) NOT NULL,
    the_other_mint VARCHAR(88) NOT NULL,
    dex_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pool_mints_pool_id_key UNIQUE (pool_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pool_mints_dex_type ON pool_mints(dex_type);
CREATE INDEX IF NOT EXISTS idx_pool_mints_mints ON pool_mints(desired_mint, the_other_mint);