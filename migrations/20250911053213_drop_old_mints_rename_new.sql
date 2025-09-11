-- Drop the old mints table
DROP TABLE IF EXISTS mints CASCADE;

-- Rename mints_v2 to mints
ALTER TABLE mints_v2 RENAME TO mints;

-- Rename all indexes to match the original naming
ALTER INDEX idx_mints_v2_symbol RENAME TO idx_mints_symbol;
ALTER INDEX idx_mints_v2_program RENAME TO idx_mints_program;

-- Rename the trigger
ALTER TRIGGER update_mints_v2_updated_at ON mints RENAME TO update_mints_updated_at;