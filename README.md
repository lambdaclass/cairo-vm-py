<div align="center">
<img src="https://i.ibb.co/mTqJq4k/cairo-rs-py2.jpg" alt="drawing" width="150"/>

### 🐍 Cairo-rs-py 🐍

EVM interpreter written in Cairo, a sort of ZK-EVM emulator, leveraging STARK proof system.

[Report Bug](https://github.com/lambdaclass/cairo-rs-py/issues/new?labels=bug&title=bug%3A+) · [Request Feature](https://github.com/lambdaclass/cairo-rs-py/issues/new?labels=enhancement&title=feat%3A+)

[![rust](https://github.com/lambdaclass/cairo-rs-py/actions/workflows/rust.yml/badge.svg)](https://github.com/lambdaclass/cairo-rs/actions/workflows/rust.yml) 
[![codecov](https://img.shields.io/codecov/c/github/lambdaclass/cairo-rs-py)](https://codecov.io/gh/lambdaclass/cairo-rs-py)
[![license](https://img.shields.io/github/license/lambdaclass/cairo-rs-py)](/LICENSE)
[![Telegram Chat][tg-badge]][tg-url]

[tg-badge]: https://img.shields.io/static/v1?color=green&logo=telegram&label=chat&style=flat&message=join
[tg-url]: https://t.me/starknet_rs

</div>


## Table of Contents

- [About](#-about)
- [Getting Started](#-getting-started)
  * [Dependencies](#dependencies)
  * [Installation](#installation)
- [Usage](#-usage)
  * [Testing](#testing)
  * [Demo](#demo)
  * [How to manually install the script dependencies](#how-to-manually-install-the-script-dependencies)
- [Benchmarking](#-benchmarking)
- [Related Projects](#-related-projects)
- [License](#%EF%B8%8F-license)

## 📖 About

`cairo-rs-py` adds Python bindings to the [cairo-rs](https://github.com/lambdaclass/cairo-rs) Cairo VM.

## 🌅 Getting Started

### Dependencies
- Rust and Cargo
- Pyenv and Python 3.9
- GMP
- make

### Installation

To set up the Python environment, and install necessary Python libraries, run `make deps`. This command builds two virtual environments, one intended for the Rust VM and the other one for the Original Python VM. It also initializes the submodules of some of the projects we integrated with. 

After setting up the environments, you can install the python binary using `maturin develop --release`.

## 🚀 Usage

After installation, you can access the cairo-rs VM from Python code. As an example, after compiling the program `array_sum` into `cairo_programs/array_sum.json`, you can run it with the VM using:

```python
import cairo_rs_py

with open(f"cairo_programs/array_sum.json") as file:
    runner = cairo_rs_py.CairoRunner(file.read(), "main", "all", False)
    runner.cairo_run(True)
```

### Testing
To run the test suite:
```bash
make full-test
```

### Demo

The `build_envs.sh` script will build two Python virtual environments:
- `cairo-lang` containing a pristine installation of `cairo-lang==0.10.3`;
- `cairo-rs-py` containing a patched installation of `cairo-lang==0.10.3` that uses `cairo-rs-py` as dependency.
It will also install the required dependencies automatically in Debian-based distributions, CentOs, Fedora and OSX. 
If you use another OS, you can check how to install them manually below.

To run the script:
```shell
./scripts/build_envs.sh
```

Both virtual environment will be created under the `/scripts` directory.

To actually use any of the implementations, you would have to activate the environment you want. For example, to use the cairo-rs-py integration you need to run:

```shell
source scripts/cairo-rs-py/bin/activate
```

After activating the cairo-rs-py virtualenv you can try out any Cairo project and it will use cairo-rs. In some cases some projects are coupled to cairo-run or need some extra patching to be able to use the Cairo-rs runner (e.g. Protostar, Zerosync).

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

## 📊 Benchmarking
To run the benchmarks of the projects we integrated with, first you need to set up the dependencies:
```bash
make benchmark-deps
```

Lastly, run make + the project you desire to try: 
```bash
benchmark-devnet
```

## 🌞 Related Projects

- [cairo-rs](https://github.com/lambdaclass/cairo-rs): A fast implementation of the Cairo VM in Rust.
- [starknet_in_rust](https://github.com/lambdaclass/starknet_in_rust): implementation of Starknet in Rust, powered by the cairo-rs VM.

## ⚖️ License

This project is licensed under the Apache 2.0 license.

See [LICENSE](/LICENSE) for more information.
  
