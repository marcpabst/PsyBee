# Visual stimuli

> **A note on coordinate systems**
>
> The coordinate system used by `psychophysics-rs` is different from the one used by most graphics libraries. In `psychophysics-rs`, the origin (0, 0) is at the center of the screen, and the x-axis increases to the right and the y-axis increases upwards. This was chosen because it aligns with how most scientists and engineers think about visual space.

There are different types of visual stimuli that can be presented to a participant. Most often, however, you will be working with `PatternStimulus`. A `PatternStimulus` is composed of a `Shape`, a `Pattern`, and a `Stroke`. The `Shape` defines the geometry of the stimulus, the `Pattern` defines the color or texture, and the `Stroke` defines the border of the stimulus. Every `PatternStimulus` needs to have a `Shape`, and either a `Pattern` or a `Stroke` (or both).

Here is an example of how to create a red circle in the center of the screen (colour handling is covered in a later chapter, but here we create a red color using the sRGB color space):

```rust
// Create a circle with a radius of 0.5, centered at (0px, 0px)
let shape = Circle::new(Point::new(0.0, 0.0), 0.5);
let pattern = UniformColor::new(SRGB::new(1.0, 0.0, 0.0));
let stimulus = PatternStimulus::new(shape, pattern, None);
```
```python
# Create a circle with a radius of 0.5, centered at (0px, 0px)
circle = psy.Circle(psy.Point(0.0, 0.0), 0.5)
red = psy.SRGB(1.0, 0.0, 0.0)
pattern = psy.UniformColor(red)
stimulus = psy.PatternStimulus(circle, pattern)
```

For convenience, Psychophysics provides a number of predefined Stimuli that you can use out of the box. For example, the previous example can be rewritten as:

```rust
let stimulus = UniformColorStimulus::new(Circle::new((0.0, 0.0), 0.5), SRGB::new(1.0, 0.0, 0.0));
```
```python
stimulus = psy.UniformColorStimulus(psy.Circle(psy.Point(0.0, 0.0), 0.5), psy.SRGB(1.0, 0.0, 0.0))
```

## Using physical units

`psychophysics-rs` allows you to specify the size of a stimulus in physical units, such as degrees of visual angle or millimeters, instead of pixels. This is useful because it allows you to create stimuli that are independent of the screen resolution and size. To do this, you need to specify the viewing distance and the screen dimensions when creating a `Window`. These values can be updated at any time using the `set_viewing_distance` and `set_screen_dimensions` methods.

All functions that accept a point of coordinates or a size of dimensions accept a `Size` type. At the time of writing, the `Size` enum has the following variants: 

- `Pixels(f32)`: The size in pixels
- `Degrees(f32)`: The size in degrees of visual angle
- `Millimeters(f32)`: The size in millimeters
- `ScreenHeight(f32)`: The size as a fraction of the screen height
- `ScreenWidth(f32)`: The size as a fraction of the screen width

You can perform basic arithmetic operations on `Size` values. For example, this is a valid way to specify a `x` coordinate 10mm to right of the left edge of the screen:

```rust
let x = Size::ScreenWidth(-0.5) + Size::Millimeters(10.0);
```
```python
x = psy.Size.ScreenWidth(-0.5) + psy.Size.Millimeters(10.0)
```
