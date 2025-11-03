#!/bin/bash
# Build all example apps

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "🔨 Building all example apps..."
echo ""

# Array to track results
declare -a results=()

# Build each example app
for example_dir in examples/*/; do
  if [ -f "$example_dir/whitehall.toml" ]; then
    example_name=$(basename "$example_dir")
    manifest_path="$example_dir/whitehall.toml"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Building: $example_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if cargo run -- build --manifest-path "$manifest_path"; then
      results+=("✅ $example_name")
      echo ""
    else
      results+=("❌ $example_name")
      echo ""
    fi
  fi
done

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
