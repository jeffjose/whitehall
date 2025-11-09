#!/bin/bash
# Build numbered example apps only (1-*, 2-*, etc.)

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "🔨 Building numbered example apps (1-*, 2-*, etc.)..."
echo ""

# Array to track results
declare -a results=()

# Build numbered examples only (directories starting with a digit)
for example_dir in examples/[0-9]*/; do
  example_name=$(basename "$example_dir")
  
  # Check if it's a project (has whitehall.toml) or a single file
  if [ -f "$example_dir/whitehall.toml" ]; then
    # It's a full project (like FFI examples)
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Building project example: $example_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if cargo run -- build "$example_dir"; then
      results+=("✅ $example_name")
      echo ""
    else
      results+=("❌ $example_name")
      echo ""
    fi
  else
    # It's a single-file example
    main_file=""
    
    # Check for main.wh in root or src/
    if [ -f "$example_dir/main.wh" ]; then
      main_file="$example_dir/main.wh"
    elif [ -f "$example_dir/src/main.wh" ]; then
      main_file="$example_dir/src/main.wh"
    fi
    
    if [ -n "$main_file" ]; then
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      echo "📦 Building example: $example_name"
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      
      if cargo run -- build "$main_file"; then
        results+=("✅ $example_name")
        echo ""
      else
        results+=("❌ $example_name")
        echo ""
      fi
    fi
  fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Build Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for result in "${results[@]}"; do
  echo "$result"
done

echo ""

# Count successes and failures
successes=$(printf '%s\n' "${results[@]}" | grep -c "^✅" || true)
failures=$(printf '%s\n' "${results[@]}" | grep -c "^❌" || true)

if [ "$failures" -eq 0 ]; then
  echo "🎉 All $successes example apps built successfully!"
  exit 0
else
  echo "⚠️  $successes succeeded, $failures failed"
  exit 1
fi
