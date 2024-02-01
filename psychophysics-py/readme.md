# Python Bindings for Psychophysics

This directory contains the Python bindings for Psychophysics. The bindings are
generated using PyO3 and the aim is to provide a Python interface that is as
"Pythonic" as possible. 

## Installation
Currently, the Python bindings are not available on PyPI. To install them, you
will need to clone the repository and build the bindings yourself (see below).

## Building

### Prerequisites
To build the Python bindings, you will need to have the following installed:
- Rust and Cargo (https://www.rust-lang.org/tools/install)
- Maturin (https://github.com/PyO3/maturin)
- Python 3.6+ (https://www.python.org/downloads/) installed in a virtual environment (using conda or venv)

### Instructions
To build the Python bindings, activate a virtual environment (e.g. using `conda activate`) and
run the following command inside this directory:
```bash
maturin develop
```
This will build the bindings and install them in the current Python environment. The
first time you run this command, it will take a while to build the bindings as Cargo
will need to download and compile all the dependencies. Subsequent builds will be
much faster.