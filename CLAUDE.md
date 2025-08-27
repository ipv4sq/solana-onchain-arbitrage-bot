# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a sophisticated Solana MEV bot and arbitrage system that monitors and executes arbitrage opportunities across
multiple DEX protocols using an onchain program. The project has evolved into a modular architecture with advanced
features including:

- **MEV Bot Integration**: Real-time monitoring of onchain MEV opportunities
- **Database Persistence**: SeaORM-based data management for pools and mints
- **Pool Abstraction**: Unified interface for all DEX protocols
- **Advanced Pipeline**: Pool indexing, swap monitoring, and trade strategy execution
- **Multi-DEX Support**: Comprehensive coverage of Solana DEX ecosystem

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
    - Mappers for converting from gRPC/RPC formats
    - Instruction analysis and metadata extraction
    - ALT (Address Lookup Table) utilities
    - Transaction simulation capabilities

- **Pool** (`convention/pool/`): Unified pool interface
    - `PoolDataLoader` trait for consistent pool data access
    - `PoolConfigInit` for pool initialization
    - DEX-specific implementations (Meteora DLMM/DAMM, Raydium CPMM, Whirlpool, Pump)
    - Account and data structures for each pool type

#### 2. **Pipeline Module** (`src/arb/pipeline/`)

Orchestrates the main business logic flow:

- **Pool Indexer** (`pipeline/pool_indexer/`):
    - Pool discovery and registration
    - Mint metadata fetching and caching
    - Database persistence of pool configurations

- **Swap Monitor** (`pipeline/swap_monitor/`): Real-time monitoring (placeholder for future implementation)

- **Trade Strategy** (`pipeline/trade_strategy/`): Arbitrage strategy execution (placeholder)

- **Uploader** (`pipeline/uploader/`): Data upload services (placeholder)

#### 3. **Database Module** (`src/arb/database/`)

SeaORM-based data persistence layer:

- **Core** (`database/core/`): Database connection management, transactions
- **Entities** (`database/entity/`):
    - `mint_record`: Token metadata and information
    - `pool_record`: Pool configurations with snapshots
- **Repositories** (`database/repositories/`): Data access patterns
- **Custom Types**: `PubkeyType` for Solana address storage

#### 4. **Global State** (`src/arb/global/`)

Shared state and utilities:

- **State Management** (`global/state/`):
    - `blockhash`: Dedicated thread for blockhash refresh (200ms intervals)
    - `rpc`: Global RPC client management
    - `mem_pool`: Memory pool for transaction management

- **Constants** (`global/constant/`):
    - MEV bot configuration
    - Well-known mint addresses
    - DEX program IDs

- **Enums** (`global/enums/`): Type-safe DEX type definitions

#### 5. **Program Module** (`src/arb/program/`)

Onchain program interaction:

- **MEV Bot** (`program/mev_bot/`):
    - Instruction building and serialization
    - Onchain monitoring with producer/consumer pattern
    - Fire module for transaction construction

#### 6. **Utility Module** (`src/arb/util/`)

Common utilities and traits:

- **Traits** (`util/traits/`):
    - `pubkey`: Extension methods for Pubkey (`.to_pubkey()`)
    - `orm`: SeaORM conversion traits
    - `signature`: Signature handling

- **Workers** (`util/worker/`): PubSub worker implementations

- **Types** (`util/types/`): Common type definitions like `MintPair`

### DEX Modules (`src/dex/`)

Each DEX has its own module with:

- Configuration structures
- Constants (program IDs, fees)
- Pool information structures
- Swap instruction builders

Supported DEXes:

- **Raydium**: AMM V4, CPMM, CLMM
- **Meteora**: DLMM, DAMM, DAMM V2
- **Orca**: Whirlpool
- **Pump.fun**: AMM
- **SolFi**: Custom pools
- **Vertigo**: CLMM

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

### Tables

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
- `created_at` (DateTime, optional)
- `updated_at` (DateTime, optional)

#### `mint_records`

- `address` (PubkeyType, PRIMARY KEY): Mint address
- `symbol` (String): Token symbol
- `decimals` (i16): Token decimals
- `program` (PubkeyType): Token program ID
- `created_at` (DateTime, optional)
- `updated_at` (DateTime, optional)

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

This is a production code and any bug may result into a leakage or loss, be VERY CAREFUL!!!

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

### Using Constants

**Never hardcode addresses** - use provided constants:

- `Mints::WSOL`, `Mints::USDC` - token mints
- `PoolPrograms::RAYDIUM_AMM`, etc - DEX program IDs
- `MevBot::PROGRAM_ID` - MEV bot constants

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
- **Database Optional**: Core functionality works without database
- **Configuration-driven**: Behavior controlled via config.toml

## Development Workflow

1. **Check compilation**: `cargo check` before any commits
2. **Run clippy**: `cargo clippy` for linting
3. **Format code**: `cargo fmt` for consistent style
4. **Test changes**: Run relevant tests with `cargo test`
5. **Monitor logs**: Use dual console/file logging for debugging
6. **Database migrations**: Always use SeaORM CLI for schema changes

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