-- Add new tables for comprehensive arbitrage tracking

-- Swap history table
CREATE TABLE IF NOT EXISTS swap_history (
    id SERIAL PRIMARY KEY,
    transaction_hash VARCHAR(88) NOT NULL,
    pool_id VARCHAR(88) NOT NULL,
    dex_type VARCHAR(50) NOT NULL,
    input_mint VARCHAR(88) NOT NULL,
    output_mint VARCHAR(88) NOT NULL,
    amount_in BIGINT NOT NULL,
    amount_out BIGINT NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    slot BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    CONSTRAINT swap_history_pool_id_fkey FOREIGN KEY (pool_id) 
        REFERENCES pool_mints(pool_id) ON DELETE CASCADE
);

-- Indexes for swap_history
CREATE INDEX idx_swap_history_tx_hash ON swap_history(transaction_hash);
CREATE INDEX idx_swap_history_pool_id ON swap_history(pool_id);
CREATE INDEX idx_swap_history_timestamp ON swap_history(timestamp DESC);
CREATE INDEX idx_swap_history_mints ON swap_history(input_mint, output_mint);

-- Arbitrage results table
CREATE TABLE IF NOT EXISTS arbitrage_results (
    id SERIAL PRIMARY KEY,
    transaction_hash VARCHAR(88) NOT NULL UNIQUE,
    input_mint VARCHAR(88) NOT NULL,
    output_mint VARCHAR(88) NOT NULL,
    input_amount BIGINT NOT NULL,
    output_amount BIGINT NOT NULL,
    profit_amount BIGINT NOT NULL,
    profit_percentage DECIMAL(10, 4) NOT NULL,
    path JSONB NOT NULL, -- Array of pool IDs
    gas_cost BIGINT NOT NULL,
    net_profit BIGINT NOT NULL,
    slot BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT
);

-- Indexes for arbitrage_results
CREATE INDEX idx_arbitrage_results_timestamp ON arbitrage_results(timestamp DESC);
CREATE INDEX idx_arbitrage_results_profit ON arbitrage_results(net_profit DESC);
CREATE INDEX idx_arbitrage_results_mints ON arbitrage_results(input_mint, output_mint);
CREATE INDEX idx_arbitrage_results_success ON arbitrage_results(success);
CREATE INDEX idx_arbitrage_results_path ON arbitrage_results USING GIN(path);

-- Pool metrics table
CREATE TABLE IF NOT EXISTS pool_metrics (
    id SERIAL PRIMARY KEY,
    pool_id VARCHAR(88) NOT NULL UNIQUE,
    dex_type VARCHAR(50) NOT NULL,
    tvl_usd DECIMAL(20, 6) NOT NULL DEFAULT 0,
    volume_24h_usd DECIMAL(20, 6) NOT NULL DEFAULT 0,
    volume_7d_usd DECIMAL(20, 6) NOT NULL DEFAULT 0,
    fee_24h_usd DECIMAL(20, 6) NOT NULL DEFAULT 0,
    apy_24h DECIMAL(10, 4) NOT NULL DEFAULT 0,
    price_impact_2_percent DECIMAL(10, 4) NOT NULL DEFAULT 0,
    swap_count_24h BIGINT NOT NULL DEFAULT 0,
    unique_traders_24h INTEGER NOT NULL DEFAULT 0,
    last_swap_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pool_metrics_pool_id_fkey FOREIGN KEY (pool_id) 
        REFERENCES pool_mints(pool_id) ON DELETE CASCADE
);

-- Indexes for pool_metrics
CREATE INDEX idx_pool_metrics_dex_type ON pool_metrics(dex_type);
CREATE INDEX idx_pool_metrics_tvl ON pool_metrics(tvl_usd DESC);
CREATE INDEX idx_pool_metrics_volume ON pool_metrics(volume_24h_usd DESC);
CREATE INDEX idx_pool_metrics_apy ON pool_metrics(apy_24h DESC);
CREATE INDEX idx_pool_metrics_updated ON pool_metrics(updated_at DESC);

-- Add update trigger for pool_metrics
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_pool_metrics_updated_at 
    BEFORE UPDATE ON pool_metrics 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Add some useful views
CREATE OR REPLACE VIEW v_active_pools AS
SELECT 
    pm.pool_id,
    pm.desired_mint,
    pm.the_other_mint,
    pm.dex_type,
    COALESCE(pmt.tvl_usd, 0) as tvl_usd,
    COALESCE(pmt.volume_24h_usd, 0) as volume_24h_usd,
    COALESCE(pmt.apy_24h, 0) as apy_24h,
    pmt.last_swap_at
FROM pool_mints pm
LEFT JOIN pool_metrics pmt ON pm.pool_id = pmt.pool_id
WHERE pmt.last_swap_at > NOW() - INTERVAL '7 days'
   OR pmt.last_swap_at IS NULL
ORDER BY pmt.tvl_usd DESC NULLS LAST;

CREATE OR REPLACE VIEW v_dex_summary AS
SELECT 
    dex_type,
    COUNT(*) as pool_count,
    SUM(tvl_usd) as total_tvl,
    SUM(volume_24h_usd) as total_volume_24h,
    AVG(apy_24h) as avg_apy
FROM pool_metrics
GROUP BY dex_type
ORDER BY total_tvl DESC;

CREATE OR REPLACE VIEW v_recent_arbitrage AS
SELECT 
    ar.*,
    jsonb_array_length(ar.path) as hop_count
FROM arbitrage_results ar
WHERE ar.timestamp > NOW() - INTERVAL '24 hours'
ORDER BY ar.net_profit DESC
LIMIT 100;