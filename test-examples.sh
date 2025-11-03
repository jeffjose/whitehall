#!/bin/bash
# Helper script to run transpiler example tests and show results

cargo test --test transpiler_examples_test tests::examples -- --nocapture 2>&1
