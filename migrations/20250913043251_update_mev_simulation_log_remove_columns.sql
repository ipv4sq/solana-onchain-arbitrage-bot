-- Remove columns that are no longer needed
ALTER TABLE mev_simulation_log
DROP COLUMN IF EXISTS minor_mint,
DROP COLUMN IF EXISTS desired_mint,
DROP COLUMN IF EXISTS profitable,
DROP COLUMN IF EXISTS profitability,
DROP COLUMN IF EXISTS return_data,
DROP COLUMN IF EXISTS units_per_byte,
DROP COLUMN IF EXISTS reason;