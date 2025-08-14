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

The implementation provides dynamic bin array calculation based on swap size:

```rust
// Calculate required bin arrays for a swap
let bin_arrays = calculate_bin_arrays_for_swap(
    &pool,
    active_bin_id,
    amount_in,
    is_a_to_b,
);
```

**Size-based logic:**
- Small swaps (< 0.1 SOL): 3 bin arrays (indices -1, 0, 1)
- Medium swaps (0.1-1 SOL): 4 bin arrays (indices -1, 0, 1, 2)
- Large swaps (≥ 1 SOL): 5 bin arrays (indices -2, -1, 0, 1, 2)

#### Bin Array PDA Derivation
```rust
let (bin_array_pda, _) = Pubkey::find_program_address(
    &[b"bin_array", pool.as_ref(), &index.to_le_bytes()],
    &METEORA_DLMM_PROGRAM_ID,
);
```

### Swap Execution

When executing a swap:
1. Determine the active bin array based on `active_id`
2. Calculate required bin arrays based on swap amount
3. Include bin arrays as remaining accounts in the swap instruction

### Usage

```rust
// Build swap accounts with specific amount
let accounts = config.build_accounts_with_amount(
    &payer,
    &input_mint,
    &output_mint,
    amount_in, // e.g., 1_485_000_000 for 1.485 SOL
)?;
```

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
- Dynamic bin array selection based on swap amount

### Example Test Output
```
=== BIN ARRAY CALCULATION TEST ===
Amount In: 1485000000 (1.485 SOL)
Active Bin ID: 200
Active Bin Array Index: 2

Generated 5 bin arrays for 1.485 SOL swap:
  [0] Index -2: 433yNSNcf1Gx9p8mWATybS81wQtjBfxmrnHpxNUzcMvU
  [1] Index -1: 69EaDEqwjBKKRFKrtRxb7okPDu5EP5nFhbuqrBtekwDg
  [2] Index 0: 5Dj2QB9BtRtWV6skbCy6eadj23h6o46CVHpLbjsCJCEB
  [3] Index 1: MrNAjbZvwT2awQDobynRrmkJStE5ejprQ7QmFXLvycq
  [4] Index 2: 9caL9WS3Y1RZ7L3wwXp4qa8hapTicbDY5GJJ3pteP7oX
```

## Constants

- **Program ID**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`
- **Event Authority**: `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6`
- **Bins per Array**: 70
- **Bin Array Seed**: `bin_array`

## Notes

- The swap amount calculation is currently simplified and doesn't implement full DLMM math
- Real implementation would need to fetch bin liquidity data and calculate exact outputs
- Fee calculation should consider both base fees and variable fees based on volatility
- Bin arrays are ordered by index, with the active bin array typically in the middle