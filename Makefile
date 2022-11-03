.PHONY: deps deps-macos deps-default-version build run check test clippy clean run-python-test  full-test

TEST_DIR=cairo_programs
TEST_FILES:=$(wildcard $(TEST_DIR)/*.cairo)
COMPILED_TESTS:=$(patsubst $(TEST_DIR)/%.cairo, $(TEST_DIR)/%.json, $(TEST_FILES))

$(TEST_DIR)/%.json: $(TEST_DIR)/%.cairo
	cairo-compile --cairo_path="$(TEST_DIR):$(BENCH_DIR)" $< --output $@

deps:
	CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib pip install fastecdsa
	pip install ecdsa fastecdsa sympy cairo-lang==0.9.1 maturin
	python3 -m venv cairo-rs-py-env
	pyenv install pypy3.7-7.3.9
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	pip install cairo_lang==0.9.1 && \
	deactivate

deps-macos:
	CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib pip install fastecdsa
	pip install ecdsa fastecdsa sympy cairo-lang==0.9.1 maturin
	python3 -m venv cairo-rs-py-env
	pyenv install pypy3.7-7.3.9
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib pip install fastecdsa && \
	pip install cairo_lang==0.9.1 && \
	deactivate

deps-default-version:
	pip install ecdsa fastecdsa sympy cairo-lang==0.9.1 maturin
	python3 -m venv cairo-rs-py-env
	. cairo-rs-py-env/bin/activate && \
	pip install cairo_lang==0.9.1 && \
	deactivate

build:
	cargo build --release

run:
	cargo run

check:
	cargo check

test: $(COMPILED_TESTS) 
	cargo test

clippy:
	cargo clippy  -- -D warnings

clean:
	rm -f $(TEST_DIR)/*.json
	rm -rf cairo-rs-py-env

run-python-test: $(COMPILED_TESTS)
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	maturin develop && \
	python3 hints_tests.py && \
	python3 prepare_os_context_test.py && \
	deactivate

full-test: test run-python-test
