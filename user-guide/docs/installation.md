## Installation (Python)

Pychophysics can be installed from PyPI using `pip`, from Conda using `conda`, or from GitHub using `pip`. If you instal from PyPI or Conda, a precompiled binary will be downloaded (if available for your platform). If you install from GitHub, the library will be compiled from source. This is also true if there are no precompiled binaries available for your platform. 

It is generally recommended to install pychophysics into a virtual environment. This can be done using `venv` or `conda`. Please refer to the [Python documentation](https://docs.python.org/3/library/venv.html) or the [Conda documentation](https://docs.conda.io/projects/conda/en/latest/user-guide/tasks/manage-environments.html) for more information.

!!! Note

    We currently provide precompiled binaries for Windows (x64), Mac (arm64), Linux (x64), iOS (arm64), and Android (arm64). If you want to compile the library from source, you will need a number of dependencies. Please refer to the [Rust installation guide](https://www.rust-lang.org/tools/install) for more information.


=== ":simple-pypi: PyPI"

    ```bash
    pip install psychophysics-py
    ```

=== ":simple-anaconda: Conda"

    ```bash
    conda install -c conda-forge psychophysics-py
    ```

=== ":fontawesome-brands-github: GitHub"

    ```bash
    pip install git+https://github.com/marcpabst/psychophysics/
    ```
**Thats it - you're all set to write your first psychophysics experiment!**

Alternatively, if you're using [Briefcase](https://beeware.org/project/projects/tools/briefcase/) or another packaging tool that supports `pyproject.toml`, you can add the following to your `pyproject.toml` file. Note that Poetry has a different format for specifying dependencies, amd you will need to refer to the [Poetry documentation](https://python-poetry.org/docs/) for more information.

```toml
[project]
dependencies = [
    "psychophysics-py"
]
```



## Installation (Rust)

Pychophysics can be installed from Crates.io using `cargo`, or from GitHub using `cargo`. The only external dependency you will need is gstreamer-1.0 (with the execption of iOS, Android, and the web, where gstreamer-assocated features are disabled).

```bash
cargo add psychophysics
```