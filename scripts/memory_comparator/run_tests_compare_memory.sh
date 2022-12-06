#!/usr/bin/env sh
# Please run this script from cairo-rs-py directory
exit_code=0
# We need to be inside starknet-devnet in order to run poetry
cd starknet-devnet
for file in test/test_*.py; do
    # Skip files that dont run entrypoints and dont produce memory and trace outputs
    if !  ([ "$file" = "test/test_estimate_fee.py" ] || [ "$file" = "test/test_postman.py" ] || [ "$file" = "test/testnet_deployment.py" ] || [ "$file" = "test/testnet_deploy.py" ] || [ "$file" = "test/test_api_specifications.py" ] || [ "$file" = "test/test_fork_cli_params.py" ] || [ "$file" = "test/test_timestamps.py" ]); then
        # Run tests in cairo-rs-py env
        . ../scripts/memory_comparator/cairo-rs-py/bin/activate
        poetry run pytest $file
        # Run tests in cairo-lang env
        . ../scripts/memory_comparator/cairo-lang/bin/activate
        poetry run pytest $file
        # Compare memory outputs
        memory_dir="memory_files"
        memory_comparator_path="../scripts/memory_comparator/memory_comparator.py"
        for mem_file in $(ls $memory_dir | grep .rs.memory$ | sed -E 's/\.rs.memory$//'); do
            if ! $memory_comparator_path $memory_dir/$mem_file.memory $memory_dir/$mem_file.rs.memory; then
                echo "Memory differs for $mem_file on test $file"
                exit_code=1
            else
                echo "Memory comparison successful for $mem_file on test $file"
            fi
        done
        # Compare trace outputs 
        trace_dir="trace_files"
        for trace_file in $(ls $trace_dir | grep .rs.trace$ | sed -E 's/\.rs.trace$//'); do
            if ! diff -q $trace_dir/$trace_file.trace $trace_dir/$trace_file.rs.trace; then
                echo "Traces differs for $trace_file on test $file"
                exit_code=1
            else
                echo "Trace comparison successful $trace_file on test $file"
            fi
        done
        # Cleanup memory files
        rm memory_files/*.memory
        # Cleanup trace files
        rm trace_files/*.trace
    fi
done
exit "${exit_code}"
