ALTER TABLE mev_simulation_log
ADD COLUMN pool_types TEXT[] NOT NULL DEFAULT '{}';