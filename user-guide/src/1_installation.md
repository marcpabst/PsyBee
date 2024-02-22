# Installation

Psychophysics can be added as a dependency by using your favorite package manager like `cargo` for Rust or `pip`/`conda` for Python. 

```bash
cargo add psychophysics
```
```bash
conda install psychophysics
```

That's it! You're ready to start using Psychophysics!

<div class="warning">
Note that there is not need to install Rust if you are using the Python bindings!

You will need to have Rust installed if you want to write your own experiments in Rust or build the Python bindings yourself.
</div>

You can install Rust by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install). After that, you can use `cargo` to create a new project:


# Installation from source

If you want to install Psychophysics from source, you can do so by cloning the repository and running the following commands:


```bash
git clone URL
cd psychophysics
cargo build
```

The Python bindings are built using `pyo3` and `maturin`. Check out the GitHub repository for more information on how to use build and test the library. Note that for some features, you may need to install additional dependencies (such as `gstreamer` for video playback).