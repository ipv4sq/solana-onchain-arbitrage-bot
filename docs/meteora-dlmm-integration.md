# Meteora DLMM Integration

## Overview

The Meteora DLMM (Discretized Liquidity Market Maker) integration provides support for swapping on Meteora's bin-based liquidity pools. This implementation includes proper bin array handling for executing swaps across liquidity bins.

## Key Components

### Pool Data Structure
The `MeteoraDlmmAccountData` struct contains:
- `active_id`: The current active bin ID where trading occurs
- `bin_step`: The price step between adjacent bins (in basis points)
- `bin_array_bitmap`: Bitmap indicating which bin arrays have liquidity
- Token mints and reserves
- Oracle and fee parameters

### Bin Arrays

Meteora DLMM uses a bin-based system where liquidity is distributed across discrete price bins:
- Each bin represents a specific price range
- Bins are grouped into arrays of 70 bins each
- Swaps may cross multiple bins depending on liquidity and size

#### Bin Array Calculation
```rust
// Each bin array contains 70 bins
const BINS_PER_ARRAY: i32 = 70;

// Calculate bin array index from bin ID
let bin_array_index = bin_id / BINS_PER_ARRAY;

// Derive bin array PDA
let (bin_array_pda, _) = Pubkey::find_program_address(
    &[b"bin_array", pool.as_ref(), &bin_array_index.to_le_bytes()],
    &METEORA_DLMM_PROGRAM_ID,
);
```

### Swap Execution

When executing a swap:
1. Determine the active bin array based on `active_id`
2. Include adjacent bin arrays for larger swaps that may cross bins
3. Pass bin arrays as remaining accounts to the swap instruction

The implementation currently includes:
- Previous bin array (index - 1)
- Current bin array (index)
- Next bin array (index + 1)

This ensures sufficient liquidity coverage for most swaps.

## Reference Implementation

The `meteora-cpi-examples` submodule contains Meteora's official CPI examples:
- Swap instruction building
- Bin array derivation helpers
- Integration tests

### Key Files
- `meteora-cpi-examples/programs/cpi-example/src/instructions/dlmm_cpi/swap.rs` - Swap CPI implementation
- `meteora-cpi-examples/programs/cpi-example/tests/integration/helpers/dlmm_pda.rs` - PDA derivation helpers
- `meteora-cpi-examples/programs/cpi-example/tests/integration/helpers/dlmm_utils.rs` - Utility functions

## Testing

The implementation includes tests that verify:
- Correct bin array PDA derivation
- Proper account structure for swap instructions
- Bin array selection based on active bin ID

## Constants

- **Program ID**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`
- **Event Authority**: `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6`
- **Bins per Array**: 70
- **Bin Array Seed**: `bin_array`

## Notes

- The swap amount calculation is currently simplified and doesn't implement full DLMM math
- Real implementation would need to fetch bin liquidity data and calculate exact outputs
- Fee calculation should consider both base fees and variable fees based on volatility