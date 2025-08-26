ALTER TABLE mev_simulation_log
DROP COLUMN IF EXISTS inner_instructions,
DROP COLUMN IF EXISTS tx_signature;
