#!/bin/sh

set -e

# This is not reaaaaally a robust way to find it, but you need to be actively
# trying to break it for this to fail :)
SCRIPT_DIR="scripts/memory_comparator"

python3.9 -m venv --upgrade-deps ${SCRIPT_DIR}/cairo-lang ${SCRIPT_DIR}/cairo-rs-py
${SCRIPT_DIR}/cairo-lang/bin/pip install cairo-lang==0.10.1
${SCRIPT_DIR}/cairo-rs-py/bin/pip install maturin==0.14.1 cairo-lang==0.10.1
${SCRIPT_DIR}/cairo-rs-py/bin/maturin build --manifest-path Cargo.toml --release --strip --interpreter 3.9 --no-default-features --features extension
${SCRIPT_DIR}/cairo-rs-py/bin/pip install target/wheels/cairo_rs_py-*.whl

${SCRIPT_DIR}/cairo-rs-py/bin/cairo-run --version
${SCRIPT_DIR}/cairo-rs-py/bin/starknet --version
