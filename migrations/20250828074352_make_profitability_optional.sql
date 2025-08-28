-- Make profitable and profitability columns nullable
ALTER TABLE mev_simulation_log
ALTER COLUMN profitable DROP NOT NULL,
ALTER COLUMN profitable SET DEFAULT NULL;