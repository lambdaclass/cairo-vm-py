## Quick and dirty script to try out `cairo-rs-py`

The `build_envs.sh` script will build two Python virtual environments:
- `cairo-lang` containing a pristine install of `cairo-lang==0.10.1`;
- `cairo-rs-py` containing a patched install of `cairo-lang==0.10.1` that uses `cairo-rs-py`, as well as said dependency.

To use it, go to the `scripts` directory and run:
```shell
sh build_envs.sh
```

The venvs will be created under said directory.

To actually use both implementations you would have to activate the environment you want. For example to use the cairo-rs-py integration:

```shell
source activate cairo-rs-py/bin/activate
```

### Requirements

The script assumes you have a Rust toolchain, Python 3.9 and the `venv` program installed.
`cairo-lang` requires the `gmp` library to build.
You can install it on Debian-based GNU/Linux distributions with:
```shell
sudo apt install -y libgmp3-dev
```

In Mac you can use HomeBrew:
```shell
brew install gmp
```

In Mac you'll also need to tell the script where to find it:
```shell
export CFLAGS=-I/opt/homebrew/opt/gmp/include LDFLAGS=-L/opt/homebrew/opt/gmp/lib
sh build_envs.sh
```
