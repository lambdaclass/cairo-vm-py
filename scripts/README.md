## Quick and dirty script to try out `cairo-rs-py`

The `build_envs.sh` script will build two Python virtual environments:
- `cairo-lang` containing a pristine install of `cairo-lang==0.10.2`;
- `cairo-rs-py` containing a patched install of `cairo-lang==0.10.2` that uses `cairo-rs-py`, as well as said dependency.

To use it, go to the `scripts` directory and run:
```shell
sh build_envs.sh
```

The venvs will be created under said directory.

### Requirements

The script assumes you have a Rust toolchain, Python 3.9 and the `venv` program installed.
