CREATE TABLE IF NOT EXISTS mev_simulation_log (
    id SERIAL PRIMARY KEY,
    minor_mint VARCHAR(44) NOT NULL,
    desired_mint VARCHAR(44) NOT NULL,
    minor_mint_sym VARCHAR(20) NOT NULL,
    desired_mint_sym VARCHAR(20) NOT NULL,
    pools TEXT[] NOT NULL,
    profitable BOOLEAN NOT NULL,
    details JSONB NOT NULL,
    profitability BIGINT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_mev_simulation_log_minor_mint ON mev_simulation_log(minor_mint);
CREATE INDEX idx_mev_simulation_log_desired_mint ON mev_simulation_log(desired_mint);
CREATE INDEX idx_mev_simulation_log_profitable ON mev_simulation_log(profitable);
CREATE INDEX idx_mev_simulation_log_created_at ON mev_simulation_log(created_at DESC);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_mev_simulation_log_updated_at BEFORE UPDATE
    ON mev_simulation_log FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();