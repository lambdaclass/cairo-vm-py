# cairo-rs-py
[![rust](https://github.com/lambdaclass/cairo-rs-py/actions/workflows/rust.yml/badge.svg)](https://github.com/lambdaclass/cairo-rs/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/lambdaclass/cairo-rs-py/branch/main/graph/badge.svg)](https://codecov.io/gh/lambdaclass/cairo-rs-py)

cairo-rs-py adds Python bindings to the [cairo-rs](https://github.com/lambdaclass/cairo-rs) Cairo VM.

## Dependencies
- Rust and Cargo
- Pyenv and Python 3.9
- GMP
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

## Script to try out `cairo-rs-py`

The `build_envs.sh` script will build two Python virtual environments:
- `cairo-lang` containing a pristine install of `cairo-lang==0.10.3`;
- `cairo-rs-py` containing a patched install of `cairo-lang==0.10.3` that uses `cairo-rs-py` as dependency.
It will also install the required dependencies automatically in Debian-based distributions, CentOs, Fedora and OSX. 
If you use another OS you can check how to install them manually below.

To run the script:
```shell
./scripts/build_envs.sh
```

Both virtualenvs will be created under the `/scripts` directory.

To actually use any of the implementations you would have to activate the environment you want. For example to use the cairo-rs-py integration you need to run:

```shell
source activate scripts/cairo-rs-py/bin/activate
```

After activating the cairo-rs-py virtualenv you can try out any Cairo project and it will use cairo-rs.

Note that the script assumes you have a Rust toolchain, Python 3.9 and the `venv` program installed.

### How to manually install the script dependencies

`cairo-lang` requires the `gmp` library to build.
You can install it on Debian-based GNU/Linux distributions with:
```shell
sudo apt install -y libgmp3-dev
```

In Mac you can use Homebrew:
```shell
brew install gmp
```

In Mac you'll also need to tell the script where to find the gmp lib:
```shell
export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
sh build_envs.sh
```
