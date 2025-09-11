-- Drop the old pools table
DROP TABLE IF EXISTS pools CASCADE;

-- Rename pools_v2 to pools
ALTER TABLE pools_v2 RENAME TO pools;

-- Rename all indexes to match the original naming
ALTER INDEX idx_pools_v2_base_mint RENAME TO idx_pools_base_mint;
ALTER INDEX idx_pools_v2_quote_mint RENAME TO idx_pools_quote_mint;
ALTER INDEX idx_pools_v2_base_quote RENAME TO idx_pools_base_quote;
ALTER INDEX idx_pools_v2_quote_base RENAME TO idx_pools_quote_base;
ALTER INDEX idx_pools_v2_dex_type RENAME TO idx_pools_dex_type;
ALTER INDEX idx_pools_v2_created_at RENAME TO idx_pools_created_at;

-- Rename the trigger
ALTER TRIGGER update_pools_v2_updated_at ON pools RENAME TO update_pools_updated_at;