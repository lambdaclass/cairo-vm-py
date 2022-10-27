.PHONY: deps, deps-macos, deps-default-version, build run check test clippy clean, run-python-test, run-python-test-macos, full-test-macos, full-test, run-python-test-default-version, ful-test-default-version

TEST_DIR=cairo_programs
TEST_FILES:=$(wildcard $(TEST_DIR)/*.cairo)
COMPILED_TESTS:=$(patsubst $(TEST_DIR)/%.cairo, $(TEST_DIR)/%.json, $(TEST_FILES))

$(TEST_DIR)/%.json: $(TEST_DIR)/%.cairo
	cairo-compile --cairo_path="$(TEST_DIR):$(BENCH_DIR)" $< --output $@

deps:
	python3 -m venv cairo-rs-py-env
	pyenv install pypy3.7-7.3.9
	export PYENV_VERSION=pypy3.7-7.3.9
	pip install cairo_lang==0.9.1

deps-macos:
	python3 -m venv cairo-rs-py-env
	pyenv install pypy3.7-7.3.9
	export PYENV_VERSION=pypy3.7-7.3.9
	CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib pip install fastecdsa
	pip install cairo_lang==0.9.1

deps-default-version:
	python3 -m venv cairo-rs-py-env
	pip install cairo_lang==0.9.1

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

run-python-test-macos: $(COMPILED_TESTS)
	. cairo-rs-py-env/bin/activate && \
	maturin develop && \
	python3 hints_tests.py && \
	deactivate

full-test-macos: deps-macos test run-python-test-macos clean

run-python-test: $(COMPILED_TESTS)
	. cairo-rs-py-env/bin/activate && \
	maturin develop && \
	python3 hints_tests.py && \
	deactivate

full-test: deps test run-python-test clean

run-python-test-default-version: $(COMPILED_TESTS)
	. cairo-rs-py-env/bin/activate && \
	maturin develop && \
	python3 hints_tests.py && \
	deactivate

full-test-default-version: deps-default-version test run-python-test-default-version clean
