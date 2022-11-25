# cairo-rs-py
[![rust](https://github.com/lambdaclass/cairo-rs-py/actions/workflows/rust.yml/badge.svg)](https://github.com/lambdaclass/cairo-rs/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/lambdaclass/cairo-rs-py/branch/main/graph/badge.svg)](https://codecov.io/gh/lambdaclass/cairo-rs-py)

cairo-rs-py adds Python bindings to the [cairo-rs](https://github.com/lambdaclass/cairo-rs) Cairo VM.

## Dependencies
- Rust
- Cargo
- PyEnv
- make

## Using cairo-rs-py
To setup the Python environment, and install necessary Python libraries, run `make deps`.

Finally, install into the python environment with `maturin develop --release`.

After that, you can access the cairo-rs VM from Python code. As an example, after compiling the program `array_sum` into `cairo_programs/array_sum.json`, you can run it with the VM using:

```python
import cairo_rs_py

with open(f"cairo_programs/array_sum.json") as file:
    runner = cairo_rs_py.CairoRunner(file.read(), "main", "all", False)
    runner.cairo_run(True)
```

## Testing
To run the test suite:
```bash
make full-test
```

## Code Coverage
Track of the project's code coverage: [Codecov](https://app.codecov.io/gh/lambdaclass/cairo-rs-py).
