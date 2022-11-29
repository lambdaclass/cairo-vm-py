#!/usr/bin/env sh
# Please run this script from cairo-rs-py directory

# We need to be inside starknet-devnet in order to run poetry
cd starknet-devnet
for file in test/test*.py; do
    # Run tests in cairo-rs-py env
    . ../scripts/memory_comparator/cairo-rs-py/bin/activate
    poetry run pytest $file
    # Run tests in cairo-lang env
    . ../scripts/memory_comparator/cairo-rs-py/bin/activate
    poetry run pytest $file
    break
done

cd ..