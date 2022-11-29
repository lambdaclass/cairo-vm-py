#!/usr/bin/env sh
# Please run this script from cairo-rs-py directory

# We need to be inside starknet-devnet in order to run poetry
cd starknet-devnet
for file in test/test*.py; do
    # Skip problematic files
    if ! ([ "$file" = "test/test_account.py" ] || [ "$file" = "test/invalid_file.py" ]); then
        # Run tests in cairo-rs-py env
        . ../scripts/memory_comparator/cairo-rs-py/bin/activate
        poetry run pytest $file
        # Run tests in cairo-lang env
        . ../scripts/memory_comparator/cairo-lang/bin/activate
        poetry run pytest $file
        break
        # Cleanup memory files
        rm memory_files/*.memory
    fi
done

cd ..