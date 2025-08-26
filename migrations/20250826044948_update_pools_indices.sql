-- Drop existing indices on base_mint and quote_mint
DROP INDEX IF EXISTS idx_pools_base_mint;
DROP INDEX IF EXISTS idx_pools_quote_mint;

-- Create new indices on base_vault and quote_vault
CREATE INDEX idx_pools_base_vault ON pools(base_vault);
CREATE INDEX idx_pools_quote_vault ON pools(quote_vault);