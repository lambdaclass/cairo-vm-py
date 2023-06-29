#!/bin/sh

OS="$(uname)"

# Install dependencies (best effort)

if [ ${OS} = "Darwin" ] ; then
    export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
fi 

set -e

. scripts/cairo-vm-py/bin/activate
maturin develop --release
cd protostar
pip install poetry
poetry install
git config --global protocol.file.allow always
cd ..
cd scripts
patch --directory cairo-vm-py/lib/python3.9/site-packages/ --strip 2 < move-to-cairo-vm-py.patch
cd ..
deactivate
. scripts/cairo-lang/bin/activate
maturin develop --release
cd protostar
pip install poetry
poetry install
git config --global protocol.file.allow always
deactivate
cd ..