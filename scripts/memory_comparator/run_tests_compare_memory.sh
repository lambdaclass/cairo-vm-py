#!/usr/bin/env sh
# Please run this script from cairo-rs-py directory
exit_code=0
# We need to be inside starknet-devnet in order to run poetry
cd starknet-devnet
for file in test/test*.py; do
    # Skip problematic files
    if ! ([ "$file" = "test/test_account.py" ] || [ "$file" = "test/test_estimate_fee.py" ] || ["$file" = "test/test_rpc_estimate_fee.py"]); then
        # Run tests in cairo-rs-py env
        . ../scripts/memory_comparator/cairo-rs-py/bin/activate
        poetry run pytest $file
        # Run tests in cairo-lang env
        . ../scripts/memory_comparator/cairo-lang/bin/activate
        poetry run pytest $file
        # Compare memory outputs
        class_hash_path="memory_files/class_hash"
        execute_entry_point_path="memory_files/execute_entry_point"
        memory_comparator_path="../tests/memory_comparator.py"
        if ! $memory_comparator_path $class_hash_path.memory $class_hash_path.rs.memory; then
            echo "Memory differs for last class_hash on test $file"
            exit_code=1
        fi
        if ! $memory_comparator_path $execute_entry_point_path.memory $execute_entry_point_path.rs.memory; then
            echo "Memory differs for last execute_entry_point on test $file"
            exit_code=1
        fi
        # Cleanup memory files
        rm memory_files/*.memory
    fi
done
cd ..
