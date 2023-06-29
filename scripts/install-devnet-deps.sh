#!/bin/sh

OS="$(uname)"

# Install dependencies (best effort)

if [ ${OS} = "Darwin" ] ; then
    export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
fi 

set -e

. scripts/cairo-vm-py/bin/activate
maturin develop --release
cd starknet-devnet
pip install poetry
poetry install
# Compile contracts in the starknet-devnet directory, it is not exclusive to one env. 
npm install --save-dev hardhat
. scripts/compile_contracts.sh
cd ..
deactivate
. scripts/cairo-lang/bin/activate
maturin develop --release
cd starknet-devnet
pip install poetry
poetry install
deactivate
cd ..