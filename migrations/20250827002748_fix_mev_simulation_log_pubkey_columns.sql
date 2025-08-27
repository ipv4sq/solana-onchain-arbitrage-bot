-- Change minor_mint and desired_mint columns from VARCHAR(44) to BYTEA
-- Since PubkeyType stores as binary data (32 bytes), we need BYTEA columns
ALTER TABLE mev_simulation_log 
    ALTER COLUMN minor_mint TYPE BYTEA USING NULL,
    ALTER COLUMN desired_mint TYPE BYTEA USING NULL;