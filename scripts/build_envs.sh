#!/bin/sh

python3.9 -m venv --upgrade-deps cairo-lang cairo-rs-py
./cairo-lang/bin/pip install cairo-lang==0.10.2
./cairo-rs-py/bin/pip install maturin==0.14.1 cairo-lang==0.10.2
./cairo-rs-py/bin/maturin build --manifest-path ../Cargo.toml --release --strip --interpreter 3.9 --no-default-features --features extension
./cairo-rs-py/bin/pip install ../target/wheels/cairo_rs_py-*.whl
patch --directory ./cairo-rs-py/lib/python3.9/site-packages/ --strip 2 < move-to-cairo-rs-py.patch

./cairo-rs-py/bin/cairo-run --version
./cairo-rs-py/bin/starknet --version
