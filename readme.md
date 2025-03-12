# :honeybee: psydk

### High-performance, low-latency, cross-platform experiment framework for the cognitive sciences.

> [!WARNING]
> This project is still in early development, and not everything is working yet. Feel free to try it out and provide feedback, but be aware that things may change rapidly and that there may be bugs. Issues and pull requests are welcome!

![PyPI - Version](https://img.shields.io/pypi/v/psydk?style=flat-square&logo=python&logoColor=%23FFFFFF&label=PyPi&labelColor=%23292929&color=%23016DAD) ![PyPI - Version](https://img.shields.io/pypi/v/psydk-py?style=flat-square&logo=anaconda&logoColor=%23FFFFFF&label=Conda&labelColor=%23292929&color=%23016DAD) ![Crates.io Version](https://img.shields.io/crates/v/psydk?style=flat-square&logo=rust&label=Crates.io&labelColor=%23292929&color=%23E43716) ![GitHub Release](https://img.shields.io/github/v/release/marcpabst/psydk?include_prereleases&style=flat-square&logo=github&logoColor=white&label=Release&labelColor=%233292929&color=%23e3e3e3) ![GitHub License](https://img.shields.io/github/license/marcpabst/psydk?style=flat-square&label=License%20&labelColor=%23292929&color=brightgreen)

psydk is a framework for neuroscience and psychology experiments. It is designed to be fast, accurate, and cross-platform.

## Features

- **Easy to use**: psydk is designed to be easy to use, with a simple API that allows you to create experiments quickly.
- **Cross-platform**: Write your experiments in **Python** and run them on desktop (**Windows**, **macOS**, **Linux**) and mobile (**iOS** and **Android**).
- **Easy installation**: `pip install psydk` and you're ready to go - no external dependencies!
- **Exact timings and low latency**: Depending on driver-support, psydk can provide `vblank` timestamps with a precision of ~50µs or better. psydk can also automatically detect dropped frames (currently only supported on `Windows/DirectX 12`).
- **Future-proof**: psydk is built on top of `wgpu`, a modern, low-level graphics API that allows to use of native graphics APIs like `DirectX 12`, `Metal`, `Vulkan`, and `OpenGL`.
- **Accurate color h


## Philosophy

Why another experiment framework? There are already many great tools out there, like `PsychoPy` and `PsychToolbox`. However, we believe that there is still room for improvement. Here are some of the key features that we think are important:

- *Future-proof*: Many existing experiment frameworks are built on top of older graphics APIs like OpenGL. While these APIs are still widely used, they are considered legacy and are being phased out in favor of newer APIs like Vulkan, Metal, and DirectX 12. As new hardware and software are released, it is important to have a modern, low-level graphics API that can take advantage of the latest features. `wgpu` is a great choice for this, as it is designed to be future-proof and to work well with modern hardware and software.

- *Standing on the shoulders of giants*: psydk is not supposed to reinvent the wheel - we want to build on top of existing tools and libraries, and provide a modern, high-performance, and easy-to-use interface to them. This will significantly reduce the amount of code that needs to be written and maintained. Where possible, improvements should be made in the underlying libraries, rather than in psydk itself. This will ensure that the improvements are available to everyone, not just to users of psydk and also decrease the maintenance burden of psydk.

- *Maintainability*: A framework such as psydk can only be valuable if it is actively maintained and developed. We believe that the best way to ensure this is to make the codebase as simple and clean as possible, so that it is easy for new contributors to get started and to make changes. Rust is a great language for this, as it enforces many best practices and has a strong emphasis on safety and performance. PyO3 is a great way to create Python bindings for Rust libraries, and it is actively maintained and developed.

- *Cross-platform*: We believe that experiments should be easy to run on any platform, without having to worry about compatibility issues. psydk is designed to work on desktop (Windows, macOS, Linux) and mobile (iOS, Android) platforms, with the same codebase. This will make it easier to run experiments on different platforms, and will also make it easier to share experiments with others. Mid-term, we also plan to support web-based experiments.

## Under the hood

psydk is built on top of `wgpu` and `winit` for the rendering and window management, and makes use of `pyo3` for the Python bindings. Depending on the backend you select, `wgpu` (through `wgpu-hal`) will call into the native graphics API (e.g. `DirectX 12`, `Metal`, `Vulkan`, `OpenGL`) to do the actual rendering. Note that both `wgpu` and `winit` are still in development and have
not yet stabilised their APIs, so we expect some changes in the future.
