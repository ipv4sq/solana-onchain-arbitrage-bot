-- Add trace column to mev_simulation_log table
ALTER TABLE mev_simulation_log 
ADD COLUMN trace JSONB;