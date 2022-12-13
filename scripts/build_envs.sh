#!/bin/sh

OS="$(uname)"

# Install dependencies (best effort)

if [ ${OS} = "Darwin" ] ; then
    brew install gmp
    export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
elif [ ${OS} = "Linux" ] ; then
    # Ubuntu/Debian
    sudo apt install -y libgmp3-dev || true
    # Fedora 
    sudo dnf -y install gmp || true
    # CentOS
    sudo yum install gmp-devel || true
    # Arch
    sudo pacman -Sy gmp || true
else 
    echo 'The gmp dependency is required in order to build the repository, please check out if you have it on your system'
fi 

set -e

#This is not reaaaaally a robust way to find it, but you need to be actively
# trying to break it for this to fail :)
SCRIPT_DIR=$(dirname $0)

python3.9 -m venv --upgrade-deps ${SCRIPT_DIR}/cairo-lang ${SCRIPT_DIR}/cairo-rs-py
${SCRIPT_DIR}/cairo-lang/bin/pip install cairo-lang==0.10.3
${SCRIPT_DIR}/cairo-rs-py/bin/pip install maturin==0.14.1 cairo-lang==0.10.3
${SCRIPT_DIR}/cairo-rs-py/bin/maturin build --manifest-path ${SCRIPT_DIR}/../Cargo.toml --release --strip --interpreter 3.9 --no-default-features --features extension
${SCRIPT_DIR}/cairo-rs-py/bin/pip install ${SCRIPT_DIR}/../target/wheels/cairo_rs_py-*.whl
patch --directory ${SCRIPT_DIR}/cairo-rs-py/lib/python3.9/site-packages/ --strip 2 < ${SCRIPT_DIR}/move-to-cairo-rs-py.patch

${SCRIPT_DIR}/cairo-rs-py/bin/cairo-run --version
${SCRIPT_DIR}/cairo-rs-py/bin/starknet --version
