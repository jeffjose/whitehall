#!/bin/bash

# Start Whitehall Playground (backend + frontend)
# This script runs both services in the foreground with interspersed output

set -e

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/tools/playground/backend"
FRONTEND_DIR="$PROJECT_ROOT/tools/playground/frontend"

echo -e "${BLUE}Starting Whitehall Playground...${NC}"
echo -e "${BLUE}Backend: http://localhost:3000${NC}"
echo -e "${BLUE}Frontend: http://localhost:8080${NC}"
echo ""

# Track if we're already cleaning up to avoid recursive trap
CLEANING_UP=0

# Function to cleanup background processes on exit
cleanup() {
    if [ $CLEANING_UP -eq 1 ]; then
        return
    fi
    CLEANING_UP=1

    echo ""
    echo -e "${BLUE}Shutting down...${NC}"

    # Kill all child processes of this script
    pkill -P $$ 2>/dev/null || true

    exit 0
}
trap cleanup INT TERM

# Start backend in background, prefix output with [BACKEND]
(
    cd "$BACKEND_DIR"
    cargo run 2>&1 | while IFS= read -r line; do
        echo -e "${GREEN}[BACKEND]${NC} $line"
    done
) &

# Start frontend in background, prefix output with [FRONTEND]
(
    cd "$FRONTEND_DIR"
    uv run -- python -m http.server 8080 2>&1 | while IFS= read -r line; do
        echo -e "${BLUE}[FRONTEND]${NC} $line"
    done
) &

# Wait for both processes
wait
