# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Solana onchain arbitrage bot that monitors and executes arbitrage opportunities across multiple DEX protocols using an onchain program. It includes advanced features like real-time pool monitoring, database persistence, and multiple subscription methods.

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

# Setup database (if using database features)
cp .env.example .env
# Configure DATABASE_URL in .env

# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
./scripts/migrate.sh
# Or manually:
# sqlx database create
# sqlx migrate run
```

## Logging

The bot logs to both console and file simultaneously:

- **Log Directory**: `logs/` (created automatically on first run)
- **Log File Format**: `bot_YYYYMMDD_HHMMSS.log` (e.g., `bot_20250817_164316.log`)
- **View Latest Logs**: Use `./tail_logs.sh` to tail the most recent log file
- **Manual Tail**: `tail -f logs/bot_*.log`
- **Log Level**: Controlled by `RUST_LOG` environment variable (default: `info`)

The `logs/` directory is in `.gitignore` so log files won't be committed to the repository.

## Architecture

### Core Components

1. **Bot Engine** (`src/bot.rs`): Main orchestrator that:
    - Creates ATAs automatically
    - Manages pool initialization for all DEXes
    - Refreshes blockhash every 10 seconds
    - Spawns concurrent tasks per token mint

2. **Arbitrage Module** (`src/arb/`): Advanced arbitrage implementation with:
    - **Chain Analysis** (`arb/chain/`): Transaction parsing and analysis
    - **Pool Management** (`arb/pool/`): DEX-specific pool implementations
    - **Global State** (`arb/global/`): Database connections and shared state
    - **Subscribers** (`arb/subscriber/`): PubSub and Yellowstone gRPC support
    - **Constants** (`arb/constant/`): DEX types, pool owners, and program IDs
    - **Utilities** (`arb/util/`): Helper functions and common operations

3. **DEX Modules** (`src/dex/`): Each DEX has its own module with specific swap instruction implementations:
    - Raydium (V4, CPMM, CLMM)
    - Meteora (DLMM, Dynamic AMM, DAMM V2)
    - Orca Whirlpool
    - Pump, SolFi, Vertigo

4. **Transaction System** (`src/transaction.rs`):
    - Builds arbitrage instructions with compute budget optimization
    - Uses Address Lookup Tables (ALTs) for transaction compression
    - Supports multi-RPC broadcasting ("spam" mode)
    - Integrates Kamino flashloans when enabled

5. **Pool Management** (`src/pools.rs`, `src/refresh.rs`):
    - Unified data structures for different pool types
    - Real-time account data refresh
    - Tracks vault accounts, authorities, and fees
    - Pool checker for validation

6. **Web Server** (`src/server.rs`): HTTP API endpoints for monitoring

7. **Database Integration**: Optional PostgreSQL support for:
    - Pool mint tracking
    - Historical data storage
    - Analytics and reporting

### Subscription Methods

The bot supports multiple ways to monitor the blockchain:

1. **PubSub** (`arb/subscriber/pubsub.rs`): WebSocket-based real-time updates
2. **Yellowstone gRPC** (`arb/subscriber/yellowstone.rs`): High-performance streaming

### Key Design Patterns

- **Configuration-driven**: All settings in `config.toml`
- **Async processing**: Heavy use of Tokio for concurrency
- **Modular DEX support**: Each DEX isolated in its own module
- **Error resilience**: Continues operation on individual failures
- **Database optional**: Can run with or without PostgreSQL

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
- Routing config with mint lists and pool addresses per DEX:
  - `pump_pool_list`
  - `raydium_pool_list`
  - `raydium_cp_pool_list`
  - `raydium_clmm_pool_list`
  - `meteora_damm_pool_list`
  - `meteora_dlmm_pool_list`
  - `meteora_damm_v2_pool_list`
  - `whirlpool_pool_list`
  - `vertigo_pool_list`
  - `lookup_table_accounts`
- RPC endpoints
- Wallet private key
- Optional: Spam mode settings, Kamino flashloan

### Development Notes

- Includes test utilities in `src/test/`
- Pool addresses must be manually configured
- Lookup table accounts required for transaction compression
- Each DEX module contains specific instruction building logic
- Transaction size optimization critical for success
- Database schema migrations may be required for updates
- Supports both mainnet and devnet configurations

## Database Management

### Initial Setup

```bash
# Install sqlx-cli (one-time setup)
cargo install sqlx-cli --no-default-features --features postgres

# Configure database connection
cp .env.example .env
# Edit .env and set DATABASE_URL=postgresql://username:password@localhost/database_name

# Create database (if not exists)
sqlx database create

# Run existing migrations
sqlx migrate run
```

### Migration Commands

```bash
# Create a new migration
sqlx migrate add <migration_name>
# This creates a new file in migrations/ directory

# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# List all applied migrations
sqlx migrate list
```

### Database Backup and Restore

```bash
# Backup data only (without schema)
source .env && pg_dump -a "$DATABASE_URL" > ~/Downloads/backup_$(date +%Y%m%d_%H%M%S).sql

# Backup full database (schema + data)
source .env && pg_dump "$DATABASE_URL" > ~/Downloads/full_backup_$(date +%Y%m%d_%H%M%S).sql

# Restore from backup
source .env && psql "$DATABASE_URL" < ~/Downloads/backup_file.sql
```

### Current Database Schema

The database currently has the following table:

- **pool_mints**: Stores DEX pool information
  - `id`: Primary key
  - `pool_id`: Unique pool identifier
  - `desired_mint`: Token mint address
  - `the_other_mint`: Paired token mint address  
  - `dex_type`: DEX protocol type (e.g., MeteoraDlmm, RaydiumV4)
  - `created_at`: Timestamp of record creation
  - `updated_at`: Timestamp of last update

### Database Usage in Code

The bot uses the database through `src/arb/global/db.rs` which provides:
- Connection pooling via sqlx
- Async database operations
- Pool mint tracking and queries

## Coding Principles
I am an experienced engineer with typescript, java, kotlin but not familiar with rust, I need you advise on best practice and help me code.
This is a production code and any bug may result into a leakage or loss, be VERY CAREFUL!!!

### General
- **ABSOLUTELY NO `///` DOC COMMENTS** - They clutter the code. Never use them.
- **NO COMMENTS** - Write self-documenting code. Only use `//` when absolutely critical for understanding.
- Think carefully and only action the specific task I have given you with the most concise and elegant solution that
  changes as little code as possible.
- When writing tests, avoid extra printing, it's difficult to follow and read
- Avoid unnecessary indents, be careful of using if Some() or match expression.

### Functional Programming Style

- **Prefer functional chaining over imperative loops** - Use iterator methods and method chaining for data transformations
- **Examples of preferred patterns**:
  - Use `filter_map` instead of `for` loops with `if let`
  - Use `fold` for accumulating values instead of mutable variables
  - Use `then_some` for conditional inclusion instead of if/else with push
  - Chain methods like `.entry().or_insert_with().and_modify()` for map operations
- **Benefits**: More expressive, eliminates mutable state, reduces boilerplate, creates composable transformations
- **Example transformation**:
  ```rust
  // Instead of:
  let mut result = HashMap::new();
  for item in items {
      if let Some(value) = process(item) {
          result.insert(item.key, value);
      }
  }
  
  // Prefer:
  let result: HashMap<_, _> = items
      .into_iter()
      .filter_map(|item| process(item).map(|value| (item.key, value)))
      .collect();
  ``` 

### Pubkey Creation

- **Always use `.to_pubkey()` helper method** instead of `Pubkey::from_str().unwrap()`
- This provides cleaner, more readable code and consistent error handling
- Example: `"So11111111111111111111111111111111111111112".to_pubkey()`

### Pump.fun Specific Notes

- **Coin Creator Fee Mechanism**: Pump.fun charges fees that go to the original token creator
- Every swap transaction must include the coin creator's vault authority account
- This is derived using PDA: `["creator_vault", coin_creator_pubkey]`
- Part of the trading fees automatically flow to the token creator's vault

- for seaorm models, you should ommit created_at updated_at fields, they are optional and should be taken care of db during inserts and update