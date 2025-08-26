ALTER TABLE mev_simulation_log
ADD COLUMN IF NOT EXISTS tx_signature VARCHAR(88),
ADD COLUMN IF NOT EXISTS tx_size INTEGER,
ADD COLUMN IF NOT EXISTS simulation_status VARCHAR(20),
ADD COLUMN IF NOT EXISTS compute_units_consumed BIGINT,
ADD COLUMN IF NOT EXISTS error_message TEXT,
ADD COLUMN IF NOT EXISTS logs TEXT[],
ADD COLUMN IF NOT EXISTS return_data JSONB,
ADD COLUMN IF NOT EXISTS inner_instructions JSONB,
ADD COLUMN IF NOT EXISTS units_per_byte BIGINT;

CREATE INDEX IF NOT EXISTS idx_mev_simulation_log_tx_signature ON mev_simulation_log(tx_signature);
CREATE INDEX IF NOT EXISTS idx_mev_simulation_log_simulation_status ON mev_simulation_log(simulation_status);