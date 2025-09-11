-- Rename symbol column to repr in mints table
ALTER TABLE mints RENAME COLUMN symbol TO repr;

-- Update index name to match new column name
ALTER INDEX idx_mints_symbol RENAME TO idx_mints_repr;