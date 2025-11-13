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

echo -e "${BLUE}Starting Whitehall Playground with auto-reload...${NC}"
echo -e "${BLUE}Backend: http://localhost:3000 (watches Rust files)${NC}"
echo -e "${BLUE}Frontend: http://localhost:8080 (refresh browser for changes)${NC}"
echo ""

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo -e "${GREEN}Installing cargo-watch for auto-reload...${NC}"
    cargo install cargo-watch
fi

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

    # Kill all descendant processes of the backend and frontend PIDs
    # This ensures cargo-watch and all its spawned processes are killed
    if [ ! -z "$BACKEND_PID" ]; then
        pkill -TERM -P $BACKEND_PID 2>/dev/null || true
        kill -TERM $BACKEND_PID 2>/dev/null || true
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        pkill -TERM -P $FRONTEND_PID 2>/dev/null || true
        kill -TERM $FRONTEND_PID 2>/dev/null || true
    fi

    # Also kill any processes using our ports
    lsof -ti:3000 | xargs -r kill -TERM 2>/dev/null || true
    lsof -ti:8080 | xargs -r kill -TERM 2>/dev/null || true

    # Give processes a moment to clean up
    sleep 0.5

    # Force kill if still running
    if [ ! -z "$BACKEND_PID" ]; then
        pkill -9 -P $BACKEND_PID 2>/dev/null || true
        kill -9 $BACKEND_PID 2>/dev/null || true
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        pkill -9 -P $FRONTEND_PID 2>/dev/null || true
        kill -9 $FRONTEND_PID 2>/dev/null || true
    fi

    # Force kill any remaining processes on the ports
    lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
    lsof -ti:8080 | xargs -r kill -9 2>/dev/null || true

    exit 0
}
trap cleanup INT TERM

# Start backend with cargo-watch for auto-reload on file changes
(
    cd "$BACKEND_DIR"
    cargo watch -x run 2>&1 | while IFS= read -r line; do
        echo -e "${GREEN}[BACKEND]${NC} $line"
    done
) &
BACKEND_PID=$!

# Start frontend in background, prefix output with [FRONTEND]
(
    cd "$FRONTEND_DIR"
    uv run -- python -m http.server 8080 2>&1 | while IFS= read -r line; do
        echo -e "${BLUE}[FRONTEND]${NC} $line"
    done
) &
FRONTEND_PID=$!

# Wait for both processes
wait
