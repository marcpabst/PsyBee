# :honeybee: PsyBee

### High-performance, low-latency, cross-platform expriment framework for the cognitive sciences.

> [!WARNING]
> This project is still in early development, and not everything is working yet.

![PyPI - Version](https://img.shields.io/pypi/v/psychophysics-py?style=flat-square&logo=python&logoColor=%23FFFFFF&label=PyPi&labelColor=%23292929&color=%23016DAD) ![PyPI - Version](https://img.shields.io/pypi/v/psychophysics-py?style=flat-square&logo=anaconda&logoColor=%23FFFFFF&label=Conda&labelColor=%23292929&color=%23016DAD) ![Crates.io Version](https://img.shields.io/crates/v/psychophysics?style=flat-square&logo=rust&label=Crates.io&labelColor=%23292929&color=%23E43716) ![GitHub Release](https://img.shields.io/github/v/release/marcpabst/psychophysics?include_prereleases&style=flat-square&logo=github&logoColor=white&label=Release&labelColor=%233292929&color=%23e3e3e3) ![GitHub License](https://img.shields.io/github/license/marcpabst/psychophysics?style=flat-square&label=License%20&labelColor=%23292929&color=brightgreen)

PsyBee is a framework for neuroscience and psychology experiments. It is designed to be fast, accurate, and cross-platform. It is still in early development, so it is not really ready for use yet.

### Why PsyBee?

-  **Easy to use:** `psybee` is designed to be easy to use. It provides a simple API for creating windows and drawing shapes, images, and text on them. It also provides a simple API for handling input (keyboard, mouse, and touch).
- **Accurate timing:** `psybee` is designed to be accurate. It uses `wgpu` for graphics, which is a low-level graphics API that allows to make use of modern grapics APIs (Vulkan, Metal, DX12, OpenGL, WebGPU, and WebGL).
- **Correct colour handling:** Colour spaces are hard. `psybee` has full support for the `palette` crate on Rust, and for the `colour` package on Python.
- **Cross-platform:** `psybee` currently supports Windows, Mac, Linux and the web. Support for Android and iOS is planned.
- **Fast and safe:** `psybee` is written in Rust, which is a modern language that is designed to be fast and memory safe.
- **Open source:** `psybee` is open source, so you can use it for free and you can contribute to it if you want to (please do!).
