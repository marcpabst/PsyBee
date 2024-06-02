# :honeybee: PsyBee

### High-performance, low-latency, cross-platform experiment framework for the cognitive sciences.

> [!WARNING]
> This project is still in early development, and not everything is working yet.

![PyPI - Version](https://img.shields.io/pypi/v/psybee?style=flat-square&logo=python&logoColor=%23FFFFFF&label=PyPi&labelColor=%23292929&color=%23016DAD) ![PyPI - Version](https://img.shields.io/pypi/v/psybee-py?style=flat-square&logo=anaconda&logoColor=%23FFFFFF&label=Conda&labelColor=%23292929&color=%23016DAD) ![Crates.io Version](https://img.shields.io/crates/v/psybee?style=flat-square&logo=rust&label=Crates.io&labelColor=%23292929&color=%23E43716) ![GitHub Release](https://img.shields.io/github/v/release/marcpabst/psybee?include_prereleases&style=flat-square&logo=github&logoColor=white&label=Release&labelColor=%233292929&color=%23e3e3e3) ![GitHub License](https://img.shields.io/github/license/marcpabst/psybee?style=flat-square&label=License%20&labelColor=%23292929&color=brightgreen)

PsyBee is a framework for neuroscience and psychology experiments. It is designed to be fast, accurate, and cross-platform.

## Features

- **Cross-platform**: Write your experiments in **Python** and run them on desktop (**Windows**, **macOS**, **Linux**) and mobile (**iOS** and **Android**).
- **Exact timings and low latency**: Depending on driver-support, PsyBee can provide `vblank` timestamps with a precision of ~50µs or better. PsyBee can also automatically detect dropped frames (currently only supported on `Windows/DirectX 12`).