# Solana Arbitrage Bot Web UI

A Next.js-based control panel for managing and monitoring the Solana arbitrage bot.

## Features

- **Configuration Editor**: Edit bot configuration with TOML syntax highlighting
- **Bot Controls**: Start, stop, and restart the bot
- **Real-time Monitoring**: View bot status and performance metrics
- **Dashboard**: Monitor pools and transaction history

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Rust server running on port 8080

### Installation

```bash
# From the root directory
npm run install:web
```

### Development

```bash
# Run both Rust server and Next.js dev server
npm run dev

# Or run them separately:
# Terminal 1 - Rust server
cargo run --release

# Terminal 2 - Next.js
cd web && npm run dev
```

The web UI will be available at http://localhost:3000

### Production Build

```bash
cd web
npm run build
npm run start
```

## API Endpoints

The Next.js server proxies requests to the Rust backend:

- `GET /api/config` - Fetch current configuration
- `POST /api/config` - Update configuration
- `POST /api/bot/start` - Start the bot
- `POST /api/bot/stop` - Stop the bot
- `GET /api/bot/status` - Get bot status

## Environment Variables

Create a `.env.local` file in the web directory:

```env
RUST_SERVER_URL=http://localhost:8080
```