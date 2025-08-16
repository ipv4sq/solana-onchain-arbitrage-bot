# Unified Transaction Data Model

## Overview

This module provides a unified data model for handling Solana transaction data from multiple sources. It solves the critical problem of reconciling different data structures between RPC JSON responses and gRPC streaming data, allowing the arbitrage bot to process transactions uniformly regardless of their source.

## Problem Statement

The arbitrage bot needs to consume transaction data from two fundamentally different sources:

1. **RPC JSON API** - Returns `EncodedConfirmedTransactionWithStatusMeta` with parsed UI-friendly formats
2. **gRPC Streaming** - Returns `TransactionUpdate` with raw binary protocol buffer data

These formats have significant structural differences:
- RPC uses base58-encoded strings; gRPC uses raw bytes
- RPC provides `UiMessage::Parsed` or `UiMessage::Raw`; gRPC has direct message structures
- RPC uses `OptionSerializer` wrappers; gRPC has direct optional fields
- Instruction formats and indexing differ between the two sources

## Solution Architecture

### Core Data Structures

```
data/
├── transaction.rs      # UnifiedTransaction - Top-level transaction wrapper
├── message.rs          # Message - Transaction message with accounts and instructions
├── instruction.rs      # Instruction & InnerInstructions - Instruction data
├── meta.rs            # TransactionMeta - Transaction metadata (fees, logs, etc.)
├── extractors.rs      # Business logic for extracting MEV and swap instructions
└── mapper/            # Conversion implementations
    ├── traits.rs      # ToUnified trait for conversion, InstructionExtractor for processing
    ├── from_rpc.rs    # RPC JSON to Unified conversion
    └── from_grpc.rs   # gRPC to Unified conversion
```

### Data Model Design

#### UnifiedTransaction
```rust
pub struct UnifiedTransaction {
    pub signature: String,           // Transaction signature (base58)
    pub slot: u64,                  // Block slot number
    pub message: Message,            // Transaction message
    pub meta: Option<TransactionMeta>, // Optional metadata
}
```

#### Message
```rust
pub struct Message {
    pub account_keys: Vec<Pubkey>,     // All account public keys
    pub recent_blockhash: String,      // Recent blockhash (base58)
    pub instructions: Vec<Instruction>, // Transaction instructions
}
```

#### Instruction
```rust
pub struct Instruction {
    pub program_id: Pubkey,           // Program being invoked
    pub accounts: Vec<AccountMeta>,   // Account inputs
    pub data: Vec<u8>,               // Raw instruction data
    pub instruction_index: usize,     // Position in transaction
}
```

#### TransactionMeta
```rust
pub struct TransactionMeta {
    pub fee: u64,                              // Transaction fee
    pub compute_units_consumed: Option<u64>,   // Compute units used
    pub log_messages: Vec<String>,             // Transaction logs
    pub inner_instructions: Vec<InnerInstructions>, // CPI instructions
    pub pre_balances: Vec<u64>,               // Account balances before
    pub post_balances: Vec<u64>,              // Account balances after
    pub err: Option<String>,                   // Error if failed
}
```

### Key Design Decisions

1. **Minimal Fields**: Only includes fields actually used by the processing pipeline
2. **Native Types**: Reuses Solana SDK types (`Pubkey`, `AccountMeta`) for compatibility
3. **Raw Data Storage**: Stores instruction data as `Vec<u8>` to avoid encoding/decoding overhead
4. **Flat Structure**: Avoids nested UI types that complicate conversion
5. **Direct Indexing**: Uses `Vec<Pubkey>` for account keys instead of string arrays

## Usage Pattern

### Converting from Different Sources

```rust
use crate::arb::chain::unfied::{ToUnified, InstructionExtractor};

// From RPC JSON
let rpc_tx: EncodedConfirmedTransactionWithStatusMeta = fetch_from_rpc().await?;
let unified_tx = rpc_tx.to_unified()?;

// From gRPC Stream
let grpc_update: TransactionUpdate = stream.next().await?;
let unified_tx = grpc_update.to_unified()?;

// Process uniformly regardless of source
let mev_instructions = unified_tx.extract_mev_instructions();
let swap_instructions = unified_tx.extract_swap_instructions();
```

### Processing Pipeline Integration

```rust
// Both data sources flow into the same processing logic
async fn process_transaction(unified_tx: UnifiedTransaction) -> Result<()> {
    // Extract MEV bot instructions
    let mev_ixs = unified_tx.extract_mev_instructions();
    
    // Extract swap instructions from inner instructions
    let swap_ixs = unified_tx.extract_swap_instructions();
    
    // Process swaps for arbitrage opportunities
    for swap in swap_ixs {
        analyze_arbitrage_opportunity(swap)?;
    }
    
    Ok(())
}
```

## Conversion Details

### RPC JSON Conversion (`mapper/from_rpc.rs`)

Handles conversion from `EncodedConfirmedTransactionWithStatusMeta`:
- Converts base58 strings to `Pubkey` types
- Handles both `UiMessage::Parsed` and `UiMessage::Raw` formats
- Unwraps `OptionSerializer` wrappers
- Decodes base58 instruction data to raw bytes
- Maps `UiInstruction` variants to unified format

### gRPC Conversion (`mapper/from_grpc.rs`)

Handles conversion from `TransactionUpdate`:
- Converts raw byte arrays to `Pubkey` types
- Reconstructs `AccountMeta` with proper signer/writable flags
- Preserves raw instruction data without encoding
- Maps yellowstone proto types to unified format
- Handles optional fields appropriately

## Benefits

1. **Single Processing Pipeline**: One set of business logic for all data sources
2. **Type Safety**: Strongly typed conversions with proper error handling
3. **Performance**: Minimizes unnecessary conversions and allocations
4. **Maintainability**: Isolates format-specific logic in conversion layer
5. **Extensibility**: Easy to add new data sources by implementing `ToUnified`
6. **Testing**: Can unit test conversions independently from business logic

## Future Enhancements

- [ ] Add support for versioned transactions
- [ ] Implement lazy loading for large transaction data
- [ ] Add caching layer for frequently accessed transactions
- [ ] Support for additional data sources (WebSocket, other gRPC providers)
- [ ] Performance optimizations for high-throughput scenarios

## Migration Guide

To migrate existing code to use the unified model:

1. Replace direct usage of `EncodedConfirmedTransactionWithStatusMeta` with `UnifiedTransaction`
2. Update extraction logic to use `InstructionExtractor` trait methods
3. Convert at data source boundaries using `to_unified()`
4. Remove format-specific processing logic from business code

## Technical Notes

- The unified model is designed to be zero-copy where possible
- Conversion happens once at data ingestion, not repeatedly during processing
- Error handling preserves original error context for debugging
- The model is forward-compatible with planned Solana transaction format changes