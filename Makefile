.PHONY: deps deps-macos deps-default-version build run check test clippy clean run-python-test full-test run-comparer-tracer compare_trace_memory compare_trace compare_memory compare_memory_devnet

TEST_DIR=cairo_programs
TEST_FILES:=$(wildcard $(TEST_DIR)/*.cairo)
COMPILED_TESTS:=$(patsubst $(TEST_DIR)/%.cairo, $(TEST_DIR)/%.json, $(TEST_FILES))
CAIRO_MEM:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.memory, $(COMPILED_TESTS))
CAIRO_TRACE:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.trace, $(COMPILED_TESTS))
CAIRO_RS_MEM:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.rs.memory, $(COMPILED_TESTS))
CAIRO_RS_TRACE:=$(patsubst $(TEST_DIR)/%.json, $(TEST_DIR)/%.rs.trace, $(COMPILED_TESTS))

BAD_TEST_DIR=cairo_programs/bad_programs
BAD_TEST_FILES:=$(wildcard $(BAD_TEST_DIR)/*.cairo)
COMPILED_BAD_TESTS:=$(patsubst $(BAD_TEST_DIR)/%.cairo, $(BAD_TEST_DIR)/%.json, $(BAD_TEST_FILES))

$(TEST_DIR)/%.json: $(TEST_DIR)/%.cairo
	cairo-compile --cairo_path="$(TEST_DIR):$(BENCH_DIR)" $< --output $@

$(TEST_DIR)/%.rs.trace $(TEST_DIR)/%.rs.memory: $(TEST_DIR)/%.json build
	python comparer_tracer.py $(*F)

$(TEST_DIR)/%.trace $(TEST_DIR)/%.memory: $(TEST_DIR)/%.json
	cairo-run --layout all --program $< --trace_file $@ --memory_file $(@D)/$(*F).memory

$(BAD_TEST_DIR)/%.json: $(BAD_TEST_DIR)/%.cairo
	cairo-compile $< --output $@
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
	cargo install cargo-tarpaulin && \
	deactivate

build:
	cargo build --release

run:
	cargo run

check:
	cargo check

coverage:
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	cargo tarpaulin --out Xml && \
	deactivate

test: $(COMPILED_TESTS) $(COMPILED_BAD_TESTS)
	cargo test

clippy:
	cargo clippy  -- -D warnings

clean:
	rm -f $(TEST_DIR)/*.json
	rm -f $(TEST_DIR)/*.memory
	rm -f $(TEST_DIR)/*.trace
	rm -rf cairo-rs-py-env
	rm -rf starknet-devnet
	rm -rf scripts/memory_comparator/cairo*

run-python-test: $(COMPILED_TESTS)
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	maturin develop && \
	python3 hints_tests.py && \
	python3 get_builtins_initial_stack.py && \
	deactivate

run-comparer-tracer: 
	PYENV_VERSION=pypy3.7-7.3.9 . cairo-rs-py-env/bin/activate && \
	maturin develop && \
	make compare_trace_memory && \
	deactivate

full-test: test run-python-test

compare_trace_memory: $(CAIRO_RS_TRACE) $(CAIRO_TRACE) $(CAIRO_RS_MEM) $(CAIRO_MEM)
	cd tests; ./compare_vm_state.sh trace memory

compare_trace: $(CAIRO_RS_TRACE) $(CAIRO_TRACE)
	cd tests; ./compare_vm_state.sh trace

compare_memory: $(CAIRO_RS_MEM) $(CAIRO_MEM)
	cd tests; ./compare_vm_state.sh memory
	
compare_memory_devnet:
# Set up the virtual envs
	scripts/memory_comparator/build_envs.sh
# Clone the starknet-devnet from github
	git clone git@github.com:Shard-Labs/starknet-devnet.git
# Set up the starknet-devnet in each env
# cairo-rs-py
	. scripts/memory_comparator/cairo-rs-py/bin/activate && \
	pip install starknet-devnet && \
	cd starknet-devnet; scripts/install_dev_tools.sh
# cairo-lang
	. scripts/memory_comparator/cairo-lang/bin/activate && \
	pip install starknet-devnet && \
	cd starknet-devnet; scripts/install_dev_tools.sh
# Create the folder where we will store the memory outputs
	cd starknet-devnet; mkdir memory_files
# Compile test files
	. scripts/memory_comparator/cairo-lang/bin/activate && \
	cd starknet-devnet; scripts/compile_contracts.sh
# Patch both envs
	patch --directory scripts/memory_comparator/cairo-rs-py/lib/python3.9/site-packages/ --strip 2 < scripts/memory_comparator/output-memory-cairo-rs-py.patch
	patch --directory scripts/memory_comparator/cairo-lang/lib/python3.9/site-packages/ --strip 2 < scripts/memory_comparator/output-memory-cairo-lang.patch
# Run each test one by one in each env and run the memory comparator
	./scripts/memory_comparator/run_tests_compare_memory.sh
