#!/bin/sh

OS="$(uname)"

# Install dependencies (best effort)

if [ ${OS} = "Darwin" ] ; then
    export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
fi 

set -e

. scripts/cairo-rs-py/bin/activate
cd starknet-devnet
pip install poetry
poetry install
npm install --save-dev hardhat
. scripts/compile_contracts.sh
cd ..
deactivate
. scripts/cairo-lang/bin/activate
cd starknet-devnet
pip install poetry
poetry install
npm install --save-dev hardhat
. scripts/compile_contracts.sh
deactivate
cd ..