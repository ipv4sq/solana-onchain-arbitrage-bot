CREATE TABLE IF NOT EXISTS pool_mints (
    id SERIAL PRIMARY KEY,
    pool_id VARCHAR(88) NOT NULL,
    desired_mint VARCHAR(88) NOT NULL,
    the_other_mint VARCHAR(88) NOT NULL,
    dex_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pool_id)
);

CREATE INDEX idx_pool_mints_mints ON pool_mints(desired_mint, the_other_mint);
CREATE INDEX idx_pool_mints_dex_type ON pool_mints(dex_type);