#!/bin/bash
# Helper script to run all example tests and show results

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Running Transpiler Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
transpiler_output=$(cargo test --test transpiler_examples_test tests::examples -- --nocapture 2>&1)
transpiler_exit=$?
echo "$transpiler_output"

# Extract test count from output (looks for "All X/Y tests passed!" or "X/Y tests passed")
# Strip ANSI color codes first for easier parsing
transpiler_count=$(echo "$transpiler_output" | sed 's/\x1b\[[0-9;]*m//g' | grep -oP '\d+/\d+(?= tests passed)' | head -1 || echo "")

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Running Pass-Through Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
passthru_output=$(cargo test --test passthru_examples_test tests::examples -- --nocapture 2>&1)
passthru_exit=$?
echo "$passthru_output"

# Extract test count from passthru output (looks for "X/Y tests passed!" or "X/Y tests passed")
# Strip ANSI color codes first for easier parsing
passthru_count=$(echo "$passthru_output" | sed 's/\x1b\[[0-9;]*m//g' | grep -oP '\d+/\d+(?= tests passed)' | head -1 || echo "")

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Running Optimization Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
optimization_output=$(cargo test --test optimization_examples_test -- --nocapture 2>&1)
optimization_exit=$?
echo "$optimization_output"

# Extract test count from optimization output (looks for "✓ X/Y tests passed!" or "X/Y tests passed")
# Strip ANSI color codes first for easier parsing
optimization_count=$(echo "$optimization_output" | sed 's/\x1b\[[0-9;]*m//g' | grep -oP '\d+/\d+(?= tests passed)' | head -1 || echo "")

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $transpiler_exit -eq 0 ]; then
    if [ -n "$transpiler_count" ]; then
        echo "✅ Transpiler examples: PASSED ($transpiler_count)"
    else
        echo "✅ Transpiler examples: PASSED"
    fi
else
    if [ -n "$transpiler_count" ]; then
        # Extract passed/total to show FAILED (X/Y)
        echo "❌ Transpiler examples: FAILED ($transpiler_count)"
    else
        echo "❌ Transpiler examples: FAILED"
    fi
fi

if [ $passthru_exit -eq 0 ]; then
    if [ -n "$passthru_count" ]; then
        echo "✅ Pass-through examples: PASSED ($passthru_count)"
    else
        echo "✅ Pass-through examples: PASSED"
    fi
else
    if [ -n "$passthru_count" ]; then
        echo "❌ Pass-through examples: FAILED ($passthru_count)"
    else
        echo "❌ Pass-through examples: FAILED"
    fi
fi

if [ $optimization_exit -eq 0 ]; then
    if [ -n "$optimization_count" ]; then
        echo "✅ Optimization examples: PASSED ($optimization_count)"
    else
        echo "✅ Optimization examples: PASSED"
    fi
else
    if [ -n "$optimization_count" ]; then
        echo "❌ Optimization examples: FAILED ($optimization_count)"
    else
        echo "❌ Optimization examples: FAILED"
    fi
fi

echo ""

# All test suites must pass
if [ $transpiler_exit -eq 0 ] && [ $passthru_exit -eq 0 ] && [ $optimization_exit -eq 0 ]; then
    echo "🎉 All example tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
