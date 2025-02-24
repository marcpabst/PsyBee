# Installation

## Using your favourite package manager

PsyBee can be installed from PyPI using `pip`, from Conda using `conda`, or from GitHub using `pip`. If you instal from PyPI or Conda, a precompiled binary wheel will be downloaded (if available for your platform). If you install from GitHub, the library will be compiled from source. This is also true if there are no precompiled binaries available for your platform. All external dependencies are included in the wheel, so you usually don't need to worry about installing them separately.

It is generally recommended to install PsyBee into a virtual environment. This can be done using `python -m venv` or `conda`. Please refer to the [Python documentation](https://docs.python.org/3/library/venv.html) or the [Conda documentation](https://docs.conda.io/projects/conda/en/latest/user-guide/tasks/manage-environments.html) for more information.

!!! Note

    We currently provide precompiled binaries for Windows (x64), Mac (arm64), Linux (x64), iOS (arm64), and Android (arm64). If you want to compile the library from source, you will need a number of dependencies. Please refer to the [Rust installation guide](https://www.rust-lang.org/tools/install) for more information.

=== ":simple-pypi: pixi"

    ```bash
    pixi add psybee
    ```

=== ":simple-pypi: PyPI"

    ```bash
    pip install psybee
    ```

=== ":simple-anaconda: Conda"

    ```bash
    conda install -c conda-forge psybee
    ```

=== ":fontawesome-brands-github: GitHub"

    ```bash
    pip install git+https://github.com/marcpabst/psybee/
    ```

**Thats it - you're all set to write your first PsyBee experiment!**
