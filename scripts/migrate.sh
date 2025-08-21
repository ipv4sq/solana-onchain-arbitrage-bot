#!/bin/bash

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL is not set. Please create a .env file with DATABASE_URL"
    exit 1
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "sqlx-cli is not installed. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run migrations
echo "Running database migrations..."
sqlx migrate run

# Check migration status
echo ""
echo "Current migration status:"
sqlx migrate info