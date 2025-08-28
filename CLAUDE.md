# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-grade Solana MEV bot and arbitrage system that monitors and executes arbitrage opportunities across
multiple DEX protocols using an onchain program. The project features a sophisticated modular architecture with advanced
caching, comprehensive logging, and robust error handling:

- **MEV Bot Integration**: Real-time monitoring and execution of onchain MEV opportunities
- **Advanced Caching**: Multi-layer cache system with TTL support (LoadingCache, PersistentCache, TTLLoadingCache)
- **Database Persistence**: SeaORM-based data management with complex simulation logging
- **Pool Abstraction**: Unified trait-based interface for 10+ DEX protocols
- **Active Pipeline**: Pool indexing, swap monitoring, price tracking, and trade strategy execution
- **Production Features**: Rate limiting, dual logging, background cache cleanup, connection pooling

**Onchain Program ID**: `MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz`

## MCP Tools Available

Claude Code has access to specialized MCP tools for Solana development:

### Solana Development Tools

- **`mcp__http-server__Solana_Expert__Ask_For_Help`**: Expert assistance for Solana development questions (how-to,
  concepts, APIs, SDKs, errors). Use for complex Solana-specific issues.
- **`mcp__http-server__Solana_Documentation_Search`**: RAG-based search across Solana ecosystem documentation. Use when
  you need up-to-date information about Solana features.
- **`mcp__http-server__Ask_Solana_Anchor_Framework_Expert`**: Specialized help for Anchor Framework development. Use for
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
cargo run --release

# Run with specific configuration
cargo run --release --bin solana-onchain-arbitrage-bot -- --config config.toml

# Setup configuration (first time only)
cp config.toml.example config.toml

# Setup database (required for new architecture)
cp .env.example .env
# Configure DATABASE_URL in .env

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

- **Pool** (`convention/pool/`): Unified pool interface with trait-based abstraction
    - **Interface** (`interface.rs`): Core traits - `PoolDataLoader`, `PoolConfigInit`, `InputAccountUtil`
    - **DEX Implementations**: 
        - `meteora_damm/` and `meteora_damm_v2/` - Meteora Dynamic AMM pools
        - `meteora_dlmm/` - Meteora Dynamic Liquidity Market Maker
        - `raydium_cpmm/` - Raydium Constant Product Market Maker
        - `pump_amm/` - Pump.fun AMM with creator fees
        - `whirlpool/` - Orca Whirlpool concentrated liquidity
    - Each implementation contains: `account.rs`, `data.rs`, `pool_config.rs`, often `test.rs`

#### 2. **Pipeline Module** (`src/arb/pipeline/`) - **ACTIVE IMPLEMENTATION**

Orchestrates the main business logic flow:

- **Pool Indexer** (`pipeline/pool_indexer/`):
    - `mev_bot/entry.rs` - **ACTIVE**: MEV bot transaction processing entry point
    - `pool_recorder.rs`, `token_recorder.rs` - Pool and token discovery/registration
    - `registrar.rs` - Pool registration logic

- **Swap Changes** (`pipeline/swap_changes/`) - **NEW MODULE**:
    - `account_monitor/` - Real-time pool account monitoring
    - `cache.rs`, `registrar.rs` - Swap event processing infrastructure

- **Trade Strategy** (`pipeline/trade_strategy/`):
    - `entry.rs` - Strategy execution framework
    - `price_tracker.rs` - **NEW**: Price tracking implementation

- **Uploader** (`pipeline/uploader/`):
    - `mev_bot/construct.rs` - MEV bot instruction construction
    - `wallet.rs` - Wallet management utilities

#### 3. **Database Module** (`src/arb/database/`)

Advanced SeaORM-based persistence with custom column types:

- **Custom Columns** (`database/columns/`):
    - `pubkey_type.rs` - Custom Solana address storage type
    - `pool_descriptor.rs` - Pool metadata descriptors
    - `cache_type_column.rs` - Cache type definitions

- **Entities** (`database/entity/`):
    - `mint_do.rs` - Token metadata (address, symbol, decimals, program)
    - `pool_do.rs` - Pool configurations with snapshots
    - `kv_cache.rs` - **NEW**: Generic key-value cache with TTL
    - `mev_simulation_log.rs` - **NEW**: Complex MEV simulation logging

- **Repositories** (`database/repositories/`):
    - `MintRecordRepository`, `PoolRecordRepository` - Core data access
    - `KvCacheRepository` - Generic cache operations
    - `MevSimulationLogRepository` - Simulation logging
    - **Built-in caching**: `POOL_CACHE`, `MINT_TO_POOLS`, `POOL_RECORDED` static caches

#### 4. **Global Module** (`src/arb/global/`)

Shared state and utilities:

- **State Management** (`global/state/`):
    - `blockhash.rs` - Dedicated thread for blockhash refresh (200ms intervals)
    - `rpc.rs` - Global RPC client management
    - `mem_pool.rs` - Memory pool for transaction management

- **Constants** (`global/constant/`):
    - `mev_bot.rs` - MEV bot program constants
    - `mint.rs` - Well-known mint addresses (WSOL, USDC, etc.)
    - `pool_program.rs` - All supported DEX program IDs
    - `token_program.rs` - SPL Token program constants

- **Enums** (`global/enums/`):
    - `dex_type.rs` - Comprehensive DEX type system (10+ DEXes)

- **Database** (`global/db.rs`) - Connection management
- **Trace** (`global/trace/`) - **NEW**: Tracing and monitoring types

#### 5. **Program Module** (`src/arb/program/`)

Onchain program interaction:

- **MEV Bot** (`program/mev_bot/`):
    - `ix.rs` - Instruction building and parsing
    - `ix_input.rs` - Input parameter structures for MEV operations

#### 6. **Utility Module** (`src/arb/util/`) - **EXTENSIVE UTILITY LIBRARY**

Production-grade utilities and abstractions:

- **Advanced Structures** (`util/structs/`):
    - `loading_cache.rs` - **LRU cache with async loader** (368 lines)
    - `persistent_cache.rs` - **Database-backed cache** with TTL support
    - `ttl_loading_cache.rs` - **TTL-aware LRU cache** (500 lines)
    - `rate_limiter.rs` - **Token bucket rate limiter** with burst capacity
    - `buffered_debouncer.rs` - Event debouncing utility
    - `lazy_cache.rs`, `lazy_arc.rs` - Lazy initialization utilities
    - `mint_pair.rs` - Token pair management

- **Traits** (`util/traits/`):
    - `pubkey.rs` - **`.to_pubkey()` extension** for string conversion
    - `orm.rs` - SeaORM conversion traits (`.to_orm()`)
    - `account_meta.rs`, `signature.rs` - Solana utilities

- **Workers** (`util/worker/`):
    - `pubsub.rs` - PubSub worker implementations

- **Other** (`util/`):
    - `logging.rs` - **Dual console/file logging** with auto-rotation
    - `macros.rs` - Utility macros
    - `cron/periodic_logger.rs` - Periodic logging utilities

#### 7. **SDK Module** (`src/arb/sdk/`)

- `yellowstone.rs` - Yellowstone gRPC integration for real-time data

### DEX Support

**Comprehensive multi-DEX coverage** with dedicated modules in `src/dex/`:

1. **Raydium**: AMM V4, CPMM, CLMM
2. **Meteora**: DLMM, DAMM, DAMM V2
3. **Orca**: Whirlpool
4. **Pump.fun**: AMM with creator fee mechanism
5. **SolFi**: Custom pools
6. **Vertigo**: CLMM
7. **Plus**: Additional/future DEX support

Each DEX module includes:
- Configuration structures
- Constants (program IDs, fees)
- Pool information structures  
- Swap instruction builders

### Configuration Structure

The `config.toml` includes:

```toml
[bot]
compute_unit_limit = 1400000

[routing]
[[routing.mint_config_list]]
mint = "..."
raydium_pool_list = ["..."]
meteora_dlmm_pool_list = ["..."]
# ... other pool lists
lookup_table_accounts = ["..."]
process_delay = 100

[rpc]
url = "$RPC_URL"  # Can use environment variables

[wallet]
private_key = "$WALLET_PRIVATE_KEY"

[spam]  # Optional
enabled = true
sending_rpc_urls = ["..."]
compute_unit_price = 1000000

[flashloan]  # Optional
enabled = false
```

## Database Schema

### Active Tables (11 migrations)

#### `pools`
- `address` (PubkeyType, PRIMARY KEY): Pool address
- `name` (String): Pool name
- `dex_type` (DexType): DEX protocol type
- `base_mint` (PubkeyType): Base token mint
- `quote_mint` (PubkeyType): Quote token mint
- `base_vault` (PubkeyType): Base token vault
- `quote_vault` (PubkeyType): Quote token vault
- `description` (JSON): Pool metadata descriptor
- `data_snapshot` (JSON): Latest pool state snapshot
- `created_at`, `updated_at` (DateTime, optional)

#### `mint_records`
- `address` (PubkeyType, PRIMARY KEY): Mint address
- `symbol` (String): Token symbol
- `decimals` (i16): Token decimals
- `program` (PubkeyType): Token program ID
- `created_at`, `updated_at` (DateTime, optional)

#### `kv_cache` - **NEW**
- `type` (CacheType): Cache type identifier
- `key` (String): Cache key
- `value` (JSON): Cached value
- `valid_until` (DateTime): TTL expiration

#### `mev_simulation_log` - **NEW**
- Complex MEV simulation tracking with:
- Mint pairs (`minor_mint`, `desired_mint` with symbols)
- Pool arrays and profitability metrics
- Simulation details (accounts, compute units, errors, logs, traces)
- Return data and units per byte fields

### Database Management

```bash
# Create new migration
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
- `PoolRecordRepository::is_pool_recorded()` - Check if pool exists
- `PoolRecordRepository::get_pool_by_address()` - Get cached pool
- `MintRecordRepository` methods - Cached mint operations

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
6. **Database migrations**: Always use sqlx CLI for schema changes
7. **Cache invalidation**: Remember to invalidate caches after updates

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

In `src/arb/database/entity/<table_name>.rs`:

- Define `Model` with `#[derive(DeriveEntityModel)]`
- Add `#[sea_orm(primary_key)]` to ID field
- Use `PubkeyType` for Solana addresses
- Use `FromJsonQueryResult` for JSON fields
- Create param structs in same file (not separate files)
- Omit `id`, `created_at`, `updated_at` from param structs

### 3. Create Repository

In `src/arb/database/repositories/<table_name>_repo.rs`:

- Use param structs for methods with many parameters
- Use `get_db()` directly (not `await`)
- For pagination: use `.paginate(db, limit).fetch_page(0).await`
- Set `NotSet` for auto fields in ActiveModel

### 4. Register Components

- Export entity in `entity/mod.rs`
- Export repository in `repositories/mod.rs`

### 5. Run Migration

```bash
sqlx migrate run
```

## Important Constants

- **WSOL Mint**: `So11111111111111111111111111111111111111112`
- **Token Programs**: SPL Token and Token-2022 supported
- **Blockhash Refresh**: Every 200ms via dedicated thread
- **Database Pool**: 100 max connections, 5 min connections
- please check compile error once you update files, unless you are told not to