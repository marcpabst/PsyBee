# psychophysics-rs - a high-performance, low-latency, cross-platform framework for psychophysics experiments

`psychophysics-rs` is a framework for writing psychophysics experiments in Rust. It is designed to be fast, accurate, and cross-platform. It is still in early development, so it is not really ready for use yet.


- **Easy to use:** `psychophysics-rs` is designed to be easy to use. It provides a simple API for creating windows and drawing shapes, images, and text on them. It also provides a simple API for handling input (keyboard, mouse, and touch).
- **Accurate timing:** `psychophysics-rs` is designed to be accurate. It uses `wgpu` for graphics, which is a low-level graphics API that allows to make use of modern grapics APIs (Vulkan, Metal, DX12, WebGPU, and WebGL).
- **Correct colour handling:** Colour spaces are hard. `psychophysics-rs` handles colour spaces correctly (including wide colour gamut support) and automatically converts between colour spaces when needed.
- **Cross-platform:** `psychophysics-rs` supports Windows, Mac, Linux, Android, iOS, and the web (with some caveats).
- **Fast and safe:** `psychophysics-rs` is written in Rust, which is a modern language that is designed to be fast and memory safe. 
- **Open source:** `psychophysics-rs` is open source, so you can use it for free and you can contribute to it if you want to.

## Features
| Feature       | Windows (Vulkan) | Windows (DX12) | Mac (Metal) | Linux (Vulkan) | Web (WebGPU) | Web (WebGL) |
|---------------|------------------|----------------|-------------|----------------|--------------|-------------|
| Accurate presentation timestamps | 🟨(1) | X | 🟨 | 🟨(1) | 🟥 | 🟥 |
| Wide colour gamut support(2) | 🟨 | X | 🟨 | X | 🟨 | X |
| Low latenty keyboard input | ✅ | ✅ | ✅ | ✅ | 🟥 | 🟥 |
| Low latency audio | X | X | X | X | 🟥 | 🟥 |

✅ = Supported, 🟨 = Work in progress, X = Currently not supported, 🟥 = Not possible due to technical limitations

(1) Requires driver support for `VK_GOOGLE_display_timing` extension

(2) Requires monitor and driver support

## Getting started

## Roadmap

- [ ] Basic graphics: 
    -  [x] Text rendering: This includes rendering text with different fonts, sizes, and colours.
    -  [x] Drawing shapes: This includes drawing rectangles, circles, and polygons.
    -  [x] Drawing images: This includes drawing images from files and drawing images from raw pixel data.
- [ ] Accurate timing:
    - [ ] (WIP upstream) Accurate presentation timestamps: This includes getting accurate presentation timestamps for each frame.
    - [ ] Accurate frame pacing to enable low-latenc rendering.
- [ ] Colour handling:
    - [ ] Support for different colour spaces through the `palette` crate.
    - [ ] Use 16-bit floating point values per colour channel with no out-of-gamut values clamping.
    - [ ] Implement correct colour space handling for all graphics APIs.
- [ ] Inputs:
    - [x] Keyboard input: This includes handling key presses and releases.
    - [ ] Mouse input: This includes handling mouse button presses and releases, mouse movement, and mouse wheel events.
    - [ ] Touch input: This includes handling touch presses and releases, touch movement, and touch gestures.
- [ ] Audio:
    - [ ] Low-latency audio playback.
    - [ ] Low-latency audio recording.
- [ ] Cross-platform support:
    - [x] Windows support.
    - [x] Mac support.
    - [x] Linux support.
    - [x] Web support.
    - [ ] Android support.
    - [ ] iOS support.


## FAQ
### Why another framework for psychophysics?

At the moment, if you need accurate timing for your psychophysics experiment, you have two options:

- You can use Psychtoolbox, which is a great framework, but it is written in Matlab, which is not a great language for writing large, complex programs. Psychtoolbox also only supports Desktop platforms (Windows, Mac, and Linux), so you can't run your experiments on mobile devices or in the browser.
- You use PsychoPy and probably end up writing a lot of custom code to get the timing right (and even then, you might not get it right). It will definitely involve a lot of trial and error (and frustration).
- You create something from scatch (the secret third thing).

### Why do psychophysics in Rust?

It is a bit of a running joke that people in the Rust community tend to be a bit evangelical about the language and overeagrly reccomend rewriting everything in Rust (so much so that there even is an acronym for this: riir - rewrite it in Rust). 

There are some solid arguments for using Rust for psychophysics experiments though:

- **It's easy to write fast and correct code:** Rust is a modern language with a lot of nice features that make it easy to write correct, fast, and (memory) safe code. Rust also has a great package manager (Cargo) and a great build system (also Cargo) that make it easy to manage dependencies and build your code.
- **Low-level hardware and system access:** If timing is important to you, access to low-level systems is also important - there are certainly ways to do this in Python or Matlab, but they usually involve writing some C/C++ code and interfacing with it. Rust is a systems language, so it is designed to be fast and to give you access to low-level systems.
- **Compiled binaries:** Rust is a compiled language, so you can compile your code into a binary and run it on any machine without having to worry about installing dependencies or having the right version of Python installed. Because Rust is a strongly and statically typed language, the compiler is very good at catching errors at compile time, which means that you can catch a lot of bugs before you even run your code.
- **You can run your experiments almost anywhere (yes, even in the browser):** WebAssembly is a Tier 1 target for Rust, so you can compile your code into a web app and run it in the browser. Most notably, both wgpu (the low-level graphics library used by psychophysics-rs) and winit (the windowing library) support WebAssembly. Rust can also easily target other platforms, which means that you can write your experiment once and run it on any platform (Windows, Mac, Linux, Android, iOS, and even the web, with some caveats).



# Colour handling (gamma correction)

If you have used PsychoPy or Psychtoolbox, you might be fammiliar with the concept of gamma correction. Gamma correction is a way of compensating for the fact that most monitors use a colour space that is known as "sRGB", which is a non-linear colour space. This means that the relationship between the RGB values that you send to the monitor and the actual brightness of the pixels on the screen is non-linear. While the standard[^1] was based on the properties of CRT monitors, it also matches how human perceive brightness (which is also non-linear). In effect, this means that if you double the brightness of a an sRGB pixel, it will **appear** twice as bright to a human observer, but the physical brightness of the pixel will **not** be twice as bright. Usually, this is not a problem, as most image data is encoded in sRGB, so the values that you send to the monitor are already in the right colour space. However, if you want to send linear colour values to the monitor (e.g. for psychophysics experiments), you need to convert them to the monitor's colour space first. This is what gamma correction does.

We handle things slightly differently in `psychophysics-rs`. Instead of gamma correcting the colour values before sending them off, all stimuli that accept colours will handle colour conversion internally and automatically. So you can, for example, specify an XYZ colours wich will be converted into sRGB. On top of that, `psychophysics-rs` allows you to specify the colour space (linear RGB or sRGB) for stimuli that do colour transformations (like creating colour gradients or gratings). Whenever graphics hardware supports it, we use 16-bit floating point values per colour channel with no out-of-gamut values clamping.

This also means that you can have different stimuli use different colour spaces, which is useful if you want to have a background that is in sRGB (e.g., an image) and a stimulus that is in linear colour space (e.g., a grating). Importantly, alpha blending is also handled correctly, so you can have transparent stimuli that are in different colour spaces. This also means that you don't have to worry about gamma correction in the traditional way, as it is handled automatically. However, most monitors are not well calibrated, so `psychophysics-rs` allows for gamma correctiing the final output, so that the colours that you see on the screen match the colours that you specified.

[^1]: The sRGB standard is defined by the International Electrotechnical Commission (IEC) in IEC 61966-2-1:1999, which is available from the IEC web site. The sRGB standard is incorporated in a number of ISO standards, including ISO 22028-1:2004 and ISO 12640-1:2004. The sRGB standard is freely available for review at http://www.color.org/srgb.pdf.

