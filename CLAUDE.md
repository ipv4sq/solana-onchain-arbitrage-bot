# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-grade Solana MEV bot and arbitrage system that monitors and executes arbitrage opportunities across
multiple DEX protocols using an onchain program. The project features a sophisticated modular architecture with advanced
caching, comprehensive logging, and robust error handling:

- **MEV Bot Integration**: Real-time monitoring and execution of onchain MEV opportunities
- **Advanced Caching**: Multi-layer cache system with TTL support (LoadingCache, PersistentCache, TTLLoadingCache)
- **Database Persistence**: SeaORM-based data management with complex simulation logging
- **DEX Abstraction**: Unified trait-based interface for multiple DEX protocols
- **Chain Subscription**: Transaction monitoring and pool discovery via Yellowstone gRPC
- **Production Features**: Rate limiting, dual logging, background cache cleanup, connection pooling
- **Supported DEXs**: Raydium (V4, CPMM, CLMM), Meteora (DLMM, DAMM, DAMM V2), Pump.fun (Classic & AMM), Orca Whirlpool,
  Solfi, Vertigo

**Onchain Program ID**: `MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz`

## MCP Tools Available

Claude Code has access to specialized MCP tools for Solana development:

### Solana Development Tools

- **`mcp__solana-mcp-server__Solana_Expert__Ask_For_Help`**: Expert assistance for Solana development questions (how-to,
  concepts, APIs, SDKs, errors). Use for complex Solana-specific issues.
- **`mcp__solana-mcp-server__Solana_Documentation_Search`**: RAG-based search across Solana ecosystem documentation. Use
  when
  you need up-to-date information about Solana features.
- **`mcp__solana-mcp-server__Ask_Solana_Anchor_Framework_Expert`**: Specialized help for Anchor Framework development.
  Use for
  Anchor-specific queries.

### Development Support Tools

- **`mcp__ide__getDiagnostics`**: Get IDE diagnostic information for code issues
- **`mcp__context7__resolve-library-id`** and **`mcp__context7__get-library-docs`**: Fetch up-to-date documentation for
  any library. Use when you need current library documentation beyond the knowledge cutoff.

**When to use MCP tools**: Prefer these tools for Solana-specific questions, current documentation needs, or when
dealing with complex Solana/Anchor concepts that require expert knowledge.

## Build and Run Commands

```bash
# Build the project
cargo build --release

# Check for compilation errors
cargo check

# Run the MEV bot listener (main entry point)
cargo run --release --bin solana-onchain-arbitrage-bot

# Run with specific configuration
cargo run --release --bin solana-onchain-arbitrage-bot -- --config config.toml

# Setup configuration (first time only)
cp config.toml.example config.toml
# Edit config.toml with your RPC URL, wallet private key, and pool configurations

# Configure DATABASE_URL in .env (PostgreSQL connection string)
# Configure GRPC_URL and GRPC_TOKEN for Yellowstone gRPC subscription

# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx migrate run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Logging

The bot logs to both console and file simultaneously:

- **Log Directory**: `logs/` (created automatically on first run)
- **Log File Format**: `bot_YYYYMMDD_HHMMSS.log` (e.g., `bot_20250823_164316.log`)
- **View Latest Logs**: Use `./tail_logs.sh` to tail the most recent log file
- **Manual Tail**: `tail -f logs/bot_*.log`
- **Log Level**: Controlled by `RUST_LOG` environment variable (default: `info,sqlx=warn`)

The `logs/` directory is in `.gitignore` so log files won't be committed to the repository.

## Modern Architecture

### Core Modules

#### 1. **Convention Module** (`src/arb/convention/`)

Provides abstraction layers for consistent interaction across different components:

- **Chain** (`convention/chain/`): Transaction and instruction parsing
    - **Mappers** (`mapper/`): Convert from gRPC/RPC formats (`from_grpc.rs`, `from_rpc.rs`)
    - **Utils** (`util/`): ALT utilities, instruction analysis, simulation, transaction handling
    - Core types: `account.rs`, `instruction.rs`, `transaction.rs`, `meta.rs`

#### 2. **DEX Module** (`src/arb/dex/`)

Unified DEX interface with trait-based abstraction:

- **Interface** (`interface.rs`): Core traits for pool operations
    - `PoolConfig` trait with `get_amount_out`, `mid_price` methods
- **Any Pool Config** (`any_pool_config.rs`): Universal pool configuration
- **DEX Implementations**:
    - `meteora_damm_v2/` - Meteora Dynamic AMM V2 with advanced curves
    - `meteora_dlmm/` - Meteora Dynamic Liquidity Market Maker
    - `raydium_cpmm/` - Raydium Constant Product Market Maker
    - `pump_amm/` - Pump.fun AMM with creator fees
    - `whirlpool/` - Orca Whirlpool concentrated liquidity
    - `meteora_damm/` - Legacy Meteora DAMM (minimal implementation)

Each implementation contains:

- `config.rs` - Pool configuration
- `pool_data.rs` - Pool data structures
- `price/` - Price calculation logic
- `misc/` - Supporting utilities

#### 3. **Pipeline Module** (`src/arb/pipeline/`)

Orchestrates the main business logic flow:

- **Chain Subscriber** (`pipeline/chain_subscriber/`):
    - `owner_account_subscriber.rs` - Monitor pool owner accounts via gRPC
    - `involved_account_subscriber.rs` - Monitor accounts involved in transactions
    - `registrar.rs` - Bootstrap chain subscription (starts both monitors)
    - `legacy/` - Legacy MEV bot transaction subscriber (currently disabled)

- **Event Processor** (`pipeline/event_processor/`):
    - `mev_bot/` - MEV bot transaction processing
    - `involved_account_processor.rs` - Process accounts in transactions
    - `mev_bot_processor.rs` - Process MEV bot transactions
    - `new_pool_processor.rs` - Handle new pool discovery
    - `owner_account_debouncer.rs` - Debounce owner account updates
    - `pool_update_processor.rs` - Process pool updates
    - `token_balance/` - Token balance tracking

- **Trade Strategy** (`pipeline/trade_strategy/`):
    - `entry.rs` - Strategy execution framework
    - `calculate_price.rs` - Price calculation utilities
    - `simulate_processor.rs` - Transaction simulation

- **Uploader** (`pipeline/uploader/`):
    - `mev_bot/` - MEV bot instruction construction
    - `wallet.rs` - Wallet management utilities
    - Transaction building and submission

#### 4. **Database Module** (`src/arb/database/`)

Advanced SeaORM-based persistence:

- **Custom Columns** (`database/columns/`):
    - `pubkey_type.rs` - Custom Solana address storage type
    - `cache_type_column.rs` - Cache type definitions

- **Entities**:
    - `mint_record/` - Token metadata (address, symbol, decimals, program)
    - `pool_record/` - Pool configurations with snapshots
    - `kv_cache/` - Generic key-value cache with TTL
    - `mev_simulation_log/` - Complex MEV simulation logging

Each entity module contains:

- `model.rs` - SeaORM model definition
- `repository.rs` - Data access layer
- Optional: `cache.rs`, `loader.rs`, `converter.rs`

#### 5. **Global Module** (`src/arb/global/`)

Shared state and utilities:

- **Client Management** (`global/client/`):
    - `rpc.rs` - Global RPC client management
    - `db.rs` - Database connection pool

- **Daemon Services** (`global/daemon/`):
    - `blockhash.rs` - Dedicated thread for blockhash refresh (200ms intervals)

- **State Management** (`global/state/`):
    - `any_pool_holder.rs` - Pool instance caching (replaces old PoolConfigCache)
    - `token_balance_holder.rs` - Token balance management

- **Constants** (`global/constant/`):
    - `mev_bot.rs` - MEV bot program constants
    - `mint.rs` - Well-known mint addresses (WSOL, USDC, etc.)
    - `pool_program.rs` - All supported DEX program IDs
    - `token_program.rs` - SPL Token program constants

- **Enums** (`global/enums/`):
    - `dex_type.rs` - Comprehensive DEX type system

- **Trace** (`global/trace/`) - Tracing and monitoring types

#### 6. **Program Module** (`src/arb/program/`)

Onchain program interaction:

- **MEV Bot** (`program/mev_bot/`):
    - `ix.rs` - Instruction building and parsing
    - `ix_input.rs` - Input parameter structures for MEV operations

#### 7. **Utility Module** (`src/arb/util/`)

Production-grade utilities and abstractions:

- **Advanced Structures** (`util/structs/`):
    - `loading_cache.rs` - LRU cache with async loader
    - `persistent_cache.rs` - Database-backed cache with TTL support
    - `ttl_loading_cache.rs` - TTL-aware LRU cache
    - `rate_limiter.rs` - Token bucket rate limiter with burst capacity
    - `buffered_debouncer.rs` - Event debouncing utility
    - `lazy_cache.rs`, `lazy_arc.rs` - Lazy initialization utilities
    - `mint_pair.rs` - Token pair management

- **Traits** (`util/traits/`):
    - `pubkey.rs` - `.to_pubkey()` extension for string conversion
    - `orm.rs` - SeaORM conversion traits (`.to_orm()`)
    - `account_meta.rs`, `signature.rs` - Solana utilities

- **Workers** (`util/worker/`):
    - `pubsub.rs` - PubSub worker implementations

- **Other** (`util/`):
    - `logging.rs` - Dual console/file logging with auto-rotation
    - `macros.rs` - Utility macros
    - `cron/periodic_logger.rs` - Periodic logging utilities

#### 8. **SDK Module** (`src/arb/sdk/`)

- `yellowstone.rs` - Yellowstone gRPC integration for real-time data

### Additional Top-Level Modules

Located in `src/` alongside the main `arb` module:

- `bot.rs` - Bot orchestration logic
- `config.rs` - Configuration management (supports environment variables with `$` prefix)
- `legacy_dex/` - Legacy DEX implementations (being phased out)
    - Contains legacy implementations for Raydium, Meteora, Pump, Whirlpool, Solfi, Vertigo
    - Includes pool fetching and checking utilities
- `pools.rs` - Pool management utilities
- `refresh.rs` - Data refresh logic
- `server.rs` - HTTP server for monitoring
- `service/` - Service layer implementations
- `transaction.rs` - Transaction utilities
- `util/` - Additional utilities
- `test/` - Test utilities and initialization
- `main.rs` - Entry point: initializes logging, database, blockhash daemon, and chain subscriber

### DEX Support

**Current DEX implementations** in `src/arb/dex/`:

1. **Meteora DLMM** (`meteora_dlmm/`) - Dynamic Liquidity Market Maker
    - Bin-based liquidity with dynamic fees
    - Price calculation from bin ID: `price = (1 + bin_step/10000)^bin_id`
    - Up to 10 bins traversal for liquidity
    - Fee: base fee + variable fee (volatility-based)
    - Program: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

2. **Meteora DAMM V2** (`meteora_damm_v2/`) - Dynamic AMM with multiple curve types
    - Supports various bonding curves
    - Advanced fee structures
    - Program: `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`

3. **Meteora DAMM** (`meteora_damm/`) - Legacy Dynamic AMM
    - Minimal implementation (being phased out)
    - Program: `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB`

4. **Raydium CPMM** (`raydium_cpmm/`) - Constant Product Market Maker
    - Standard x*y=k AMM
    - Configurable fees via AMM config
    - Program: `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C`

5. **Pump.fun AMM** (`pump_amm/`) - Token launch AMM
    - Creator fee mechanism (fees go to token creator)
    - Bonding curve for fair launches
    - Creator vault PDA: `["creator_vault", coin_creator_pubkey]`
    - Program: `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`

6. **Orca Whirlpool** (`whirlpool/`) - Concentrated liquidity
    - Tick-based liquidity
    - Position management
    - Program: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

**Legacy DEX implementations** (in `src/legacy_dex/`, being phased out):

- Raydium V4 (`675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`)
- Raydium CLMM (`CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`)
- Pump.fun Classic (`6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`)
- Solfi (`SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe`)
- Vertigo (`vrTGoBuy5rYSxAfV3jaRJWHH6nN9WK4NRExGxsk1bCJ`)

### Configuration Structure

The `config.toml` includes:

```toml
[bot]
compute_unit_limit = 600000  # Max compute units per transaction

[routing]
[[routing.mint_config_list]]
mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"  # Token mint address
pump_pool_list = ["..."]           # Pump.fun AMM pools
raydium_pool_list = ["..."]        # Raydium V4 pools
meteora_damm_pool_list = []        # Meteora DAMM pools
meteora_dlmm_pool_list = ["..."]   # Meteora DLMM pools
meteora_damm_v2_pool_list = []     # Meteora DAMM V2 pools
whirlpool_pool_list = ["..."]      # Orca Whirlpool pools
raydium_clmm_pool_list = ["..."]   # Raydium CLMM pools
raydium_cp_pool_list = []          # Raydium CPMM pools
solfi_pool_list = []                # Solfi pools
vertigo_pool_list = []              # Vertigo pools
lookup_table_accounts = ["..."]    # Address lookup tables for transaction size optimization
process_delay = 400                 # Delay in ms between processing

[rpc]
url = "$RPC_URL"  # Can use environment variables with $ prefix

[wallet]
private_key = "$WALLET_PRIVATE_KEY"  # Supports env vars

[spam]  # Optional - for transaction spamming
enabled = true
sending_rpc_urls = ["..."]  # Multiple RPC endpoints for redundancy
compute_unit_price = 1000
max_retries = 3

[flashloan]  # Optional - flashloan support
enabled = true
```

### Environment Variables (.env)

```bash
# Database connection (PostgreSQL)
DATABASE_URL=postgresql://username:password@localhost/database_name

# Yellowstone gRPC for chain subscription
GRPC_URL=https://your-grpc-endpoint.com
GRPC_TOKEN=your-authentication-token

# Optional: Override MEV program ID
# MEV_PROGRAM_ID=MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz
```

## Database Schema

### Active Tables (16 migrations)

#### `pools`

- `address` (BYTEA, PRIMARY KEY): Pool address
- `name` (VARCHAR): Pool name
- `dex_type` (VARCHAR): DEX protocol type
- `base_mint` (BYTEA): Base token mint
- `quote_mint` (BYTEA): Quote token mint
- `description` (JSONB): Pool metadata descriptor
- `data_snapshot` (JSONB): Latest pool state snapshot
- `created_at`, `updated_at` (TIMESTAMPTZ)

Note: `base_vault` and `quote_vault` columns were dropped in migration 20250830055911
Indexes: compound index on (base_mint, quote_mint), individual indexes on base_mint and quote_mint

#### `mint_records`

- `address` (BYTEA, PRIMARY KEY): Mint address
- `symbol` (VARCHAR): Token symbol
- `decimals` (SMALLINT): Token decimals
- `program` (BYTEA): Token program ID
- `created_at`, `updated_at` (TIMESTAMPTZ)

#### `kv_cache`

- `type` (VARCHAR): Cache type identifier
- `key` (VARCHAR): Cache key
- `value` (JSONB): Cached value
- `valid_until` (TIMESTAMPTZ): TTL expiration
- PRIMARY KEY: (type, key)

#### `mev_simulation_log`

Complex MEV simulation tracking with:

- Transaction details (signature, compute units, errors)
- Mint pairs (`minor_mint`, `desired_mint` with symbols)
- Pool arrays and pool types
- Profitability metrics
- Simulation logs and traces
- Return data and reason fields

### Database Management

```bash
# Create new migration
sqlx migrate add <migration_name>

# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Create database (if not exists)
sqlx database create

# Drop database
sqlx database drop
```

## Coding Principles

This is production code where bugs can result in financial loss. Exercise extreme caution!

### CRITICAL: Fix Forward, Never Delete

**NEVER remove code to fix compilation errors** - Always fix forward:

- When encountering compilation errors, understand the intent and fix the actual issue
- Don't delete functionality - fix types, lifetimes, or logic issues instead
- If something doesn't compile, make it compile by fixing it, not removing it

### Code Hygiene

- **NO COMMENTS**: Write self-documenting code. Comments are code smell.
    - Exception: Use `//` ONLY when the code does something non-obvious that cannot be made clear through naming
- **NO DOC COMMENTS**: Never use `///`. They clutter the code.
- **NO DEBUG LOGS**: Never use `debug!()` macro. Use `info!()`, `warn!()`, or `error!()` only
    - Remove ALL debug `println!`, `dbg!()` after debugging
    - Production code should only log critical events via proper logging framework
    - Tests should only assert, never print (unless actively debugging, then remove)
- **NO EXPLANATORY TEST MESSAGES**: Assertions should be clear without messages
    - Bad: `assert_eq!(x, 5, "x should be 5 because...")`
    - Good: `assert_eq!(x, 5)`
- **CLEAN COMMITS**: Never commit debugging artifacts, commented-out code, or TODO comments

### General

- Think carefully and only action the specific task with the most concise and elegant solution
- Change as little code as possible
- Avoid unnecessary indents, be careful of using if Some() or match expressions

### Early Returns

**Use early returns** to avoid deep nesting:

- Handle edge cases first, main logic last

### Functional Programming Style

**STRONGLY PREFER functional chaining** - Transform data through clear pipeline steps.

#### Core Principles

- **Each line = one transformation** - Complex logic becomes a readable sequence
- **No mutable state** - Use `filter_map`, `fold`, `collect()` instead of loops and mutations
- **Chain everything** - `.map()`, `.filter()`, `.collect()` create self-documenting pipelines

#### Example: Good vs Bad

```rust
// GOOD: Clear transformation pipeline
let result = items
.into_iter()
.filter_map( | item| process(item).map( | v| (item.key, v)))
.collect();

// BAD: Imperative with mutable state
let mut result = HashMap::new();
for item in items {
if let Some(value) = process(item) {
result.insert(item.key, value);
}
}
```

#### Key Patterns

- `filter_map` > `for` loops with `if let`
- `fold`/`reduce` > mutable accumulator variables
- `collect()` > manual HashMap/Vec construction
- `try_join_all` for parallel async operations
- Chain `.entry().or_insert().and_modify()` for map updates

### Pubkey Creation

- **Always use `.to_pubkey()` helper method** instead of `Pubkey::from_str().unwrap()`
- This provides cleaner, more readable code and consistent error handling
- Example: `"So11111111111111111111111111111111111111112".to_pubkey()`

### Using Constants and Utilities

**Never hardcode addresses** - use provided constants:

- `Mints::WSOL`, `Mints::USDC` - token mints
- `PoolPrograms::RAYDIUM_AMM`, etc - DEX program IDs
- `MevBot::PROGRAM_ID` - MEV bot constants

**Use built-in caching**:

- Database-backed caches in repositories
- LoadingCache for frequently accessed data
- TTLLoadingCache for time-sensitive data

### Meteora DLMM Specific Notes

Located in `src/arb/dex/meteora_dlmm/price/best_effort.rs`:

- **Price Calculation**: Always calculate from bin ID: `price = (1 + bin_step/10000)^bin_id`
- **Bin Structure**: 70 bins per array, bin array index = bin_id / 70
- **Bin Traversal**:
    - X→Y swap: Move to lower bin IDs (bins with Y liquidity)
    - Y→X swap: Move to higher bin IDs (bins with X liquidity)
- **Common Issues**:
    - Stored price in bin may not match calculated price - use calculated
    - Must check output liquidity only, not input liquidity
    - Price represents Y per X in lamport units

### Pump.fun Specific Notes

- **Coin Creator Fee Mechanism**: Pump.fun charges fees that go to the original token creator
- Every swap transaction must include the coin creator's vault authority account
- This is derived using PDA: `["creator_vault", coin_creator_pubkey]`
- Part of the trading fees automatically flow to the token creator's vault

### SeaORM Models

- For SeaORM models, omit `created_at` and `updated_at` fields in insert/update operations
- These fields are optional and handled automatically by the database
- Use `.to_orm()` trait method for converting Pubkey to database-compatible format

## Important Instructions

- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary for achieving your goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested

## Key Design Patterns

- **Async-first**: All operations use Tokio runtime
- **Error Resilience**: Use `Result<T>` everywhere, handle errors gracefully
- **Modular Architecture**: Each component has clear boundaries
- **Trait-based Abstraction**: Use traits for cross-DEX compatibility
- **Multi-layer Caching**: LoadingCache, PersistentCache, TTLLoadingCache
- **Configuration-driven**: Behavior controlled via config.toml
- **Production Monitoring**: Comprehensive logging and simulation tracking

## Development Workflow

1. **Check compilation**: `cargo check` before any commits
2. **Run clippy**: `cargo clippy` for linting
3. **Format code**: `cargo fmt` for consistent style
4. **Test changes**: Run relevant tests with `cargo test`
5. **Monitor logs**: Use dual console/file logging (`logs/bot_*.log`)
    - Use `./tail_logs.sh` to tail the latest log file
    - Logs rotate automatically with timestamp in filename
6. **Database migrations**: Always use sqlx CLI for schema changes
7. **Cache invalidation**: Remember to invalidate caches after updates
8. **Chain Subscription**: Bot uses two monitors via gRPC:
    - Owner account monitor - tracks pool owner accounts
    - Involved account monitor - tracks accounts in transactions

## Creating Database Tables

When creating a new database table, follow this pattern:

### 1. Create Migration

```bash
sqlx migrate add create_<table_name>_table
```

- Include indexes for frequently queried columns
- Add `created_at` and `updated_at` with default timestamps
- Create update trigger for `updated_at`

### 2. Create Entity

In `src/arb/database/<table_name>/model.rs`:

- Define `Model` with `#[derive(DeriveEntityModel)]`
- Add `#[sea_orm(primary_key)]` to ID field
- Use `PubkeyType` for Solana addresses
- Use `FromJsonQueryResult` for JSON fields
- Create param structs in same module
- Omit `id`, `created_at`, `updated_at` from param structs

### 3. Create Repository

In `src/arb/database/<table_name>/repository.rs`:

- Use param structs for methods with many parameters
- Use `get_db()` from `global::client::db` directly (not `await`)
- For pagination: use `.paginate(db, limit).fetch_page(0).await`
- Set `NotSet` for auto fields in ActiveModel

### 4. Register Components

- Export in `database/mod.rs`
- Create module directory with `mod.rs`, `model.rs`, `repository.rs`

### 5. Run Migration

```bash
sqlx migrate run
```

## Important Constants

- **WSOL Mint**: `So11111111111111111111111111111111111111112`
- **USDC Mint**: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- **Token Programs**: SPL Token and Token-2022 supported
- **Blockhash Refresh**: Every 200ms via dedicated thread (`global::daemon::blockhash`)
- **Database Pool**: Managed via `global::client::db` (SeaORM connection pool)
- **RPC Client**: Access via `crate::arb::global::client::rpc::rpc_client()`
- **MEV Bot Program**: `MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz`

# Important Instruction Reminders

- If you are creating a tokio test, it will be good to call src/arb/global/client/db.rs:32 must_init_db() to init db
- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary for achieving your goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested
- Check compile errors once you update files, unless told not to