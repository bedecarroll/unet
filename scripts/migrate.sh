#!/bin/bash

# Migration helper script for μNet
# Usage: ./scripts/migrate.sh [up|down|fresh|reset|status] [DATABASE_URL]

set -e

COMMAND=${1:-"up"}
DATABASE_URL=${2:-"sqlite:./unet.db?mode=rwc"}

echo "Running migration command: $COMMAND"
echo "Database URL: $DATABASE_URL"

cd "$(dirname "$0")/.."

case $COMMAND in
    "up")
        echo "Running all pending migrations..."
        DATABASE_URL="$DATABASE_URL" cargo run --package migration -- up
        ;;
    "down")
        echo "Rolling back one migration..."
        DATABASE_URL="$DATABASE_URL" cargo run --package migration -- down
        ;;
    "fresh")
        echo "Rolling back all migrations and running them again..."
        DATABASE_URL="$DATABASE_URL" cargo run --package migration -- fresh
        ;;
    "reset")
        echo "Rolling back all migrations..."
        DATABASE_URL="$DATABASE_URL" cargo run --package migration -- reset
        ;;
    "status")
        echo "Checking migration status..."
        DATABASE_URL="$DATABASE_URL" cargo run --package migration -- status
        ;;
    *)
        echo "Unknown command: $COMMAND"
        echo "Available commands: up, down, fresh, reset, status"
        exit 1
        ;;
esac

echo "Migration command completed successfully!"