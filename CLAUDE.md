# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Solana onchain arbitrage bot that demonstrates how to parse pools from multiple DEX protocols and execute
arbitrage trades using an onchain program. It's a reference implementation for advanced users, not a production-ready
bot.

**Onchain Program ID**: `MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz`

## Build and Run Commands

```bash
# Build the project
cargo build --release

# Run the bot with configuration
cargo run --release --bin solana-onchain-arbitrage-bot -- --config config.toml

# Check for compilation errors
cargo check

# Setup configuration (first time only)
cp config.toml.example config.toml
```

## Architecture

### Core Components

1. **Bot Engine** (`src/bot.rs`): Main orchestrator that:
    - Creates ATAs automatically
    - Manages pool initialization for all DEXes
    - Refreshes blockhash every 10 seconds
    - Spawns concurrent tasks per token mint

2. **DEX Modules** (`src/dex/`): Each DEX has its own module with specific swap instruction implementations:
    - Raydium (V4, CPMM, CLMM)
    - Meteora (DLMM, Dynamic AMM, DAMM V2)
    - Orca Whirlpool
    - Pump, SolFi, Vertigo

3. **Transaction System** (`src/transaction.rs`):
    - Builds arbitrage instructions with compute budget optimization
    - Uses Address Lookup Tables (ALTs) for transaction compression
    - Supports multi-RPC broadcasting ("spam" mode)
    - Integrates Kamino flashloans when enabled

4. **Pool Management** (`src/pools.rs`, `src/refresh.rs`):
    - Unified data structures for different pool types
    - Real-time account data refresh
    - Tracks vault accounts, authorities, and fees

### Key Design Patterns

- **Configuration-driven**: All settings in `config.toml`
- **Async processing**: Heavy use of Tokio for concurrency
- **Modular DEX support**: Each DEX isolated in its own module
- **Error resilience**: Continues operation on individual failures

### Transaction Flow

1. Bot monitors configured token mints
2. Refreshes pool data from all configured DEXes
3. Calculates optimal arbitrage amounts
4. Builds transaction with:
    - Compute budget instructions
    - Optional flashloan borrow
    - Swap instructions for each DEX
    - Optional flashloan repay
5. Sends transaction through configured RPCs

### Important Constants

- **WSOL Mint**: `So11111111111111111111111111111111111112`
- **Token Programs**: Supports both SPL Token and Token 2022
- **Max Compute Units**: Configurable via `compute_unit_limit`

### Configuration Structure

The `config.toml` must include:

- Bot settings (compute limits, delays)
- Routing config with mint lists and pool addresses per DEX
- RPC endpoints
- Wallet private key
- Optional: Spam mode settings, Kamino flashloan

### Development Notes

- No formal test suite exists (demo implementation)
- Pool addresses must be manually configured
- Lookup table accounts required for transaction compression
- Each DEX module contains specific instruction building logic
- Transaction size optimization critical for success

## Coding Principles
I am an experienced engineer with typescript, java, kotlin but not familiar with rust, I need you advise on best practice and help me code.
This is a production code and any bug may result into a leakage or loss, be VERY CAREFUL!!!

### General
- DO NOT USE /// for comments, it looks annoying, normal // will be good
- Think carefully and only action the specific task I have given you with the most concise and elegant solution that
  changes as little code as possible.
- Avoid comments, only add comments when absolutely necessary, your code is kind of messy because you add too many comments.
- When writing tests, avoid extra printing, it's difficult to follow and read
- Avoid necessary indents, be caseful of using if Some() or match expression. 

### Pubkey Creation

- **Always use `.to_pubkey()` helper method** instead of `Pubkey::from_str().unwrap()`
- This provides cleaner, more readable code and consistent error handling
- Example: `"So11111111111111111111111111111111111111112".to_pubkey()`

### Pump.fun Specific Notes

- **Coin Creator Fee Mechanism**: Pump.fun charges fees that go to the original token creator
- Every swap transaction must include the coin creator's vault authority account
- This is derived using PDA: `["creator_vault", coin_creator_pubkey]`
- Part of the trading fees automatically flow to the token creator's vault
