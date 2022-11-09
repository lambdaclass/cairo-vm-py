#!/bin/bash
echo "Running hint tests"
python3 hints_tests.py 
python3 tests/get_builtins_initial_stack.py 

echo "Running prepare_os_context integration tests"
python3 tests/prepare_os_context.py  