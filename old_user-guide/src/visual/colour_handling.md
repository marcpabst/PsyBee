# Colour handling and gamma correction

You might be surprised to learn that there is no such thing as a simple RGB colour in `psychophysics`. The reason for this is that many different colour spaces exist that can be represented as RGB. And since there is simply no way to know what RGB values mean without knowing the colour space, `psychophysics` requires you to specify the colour space when creating a colour. This removes ambiguity and ensures that you always know what you are doing.

## Colours in `psychophysics`

### A note on colour management

Within `psychophysics`'s internal graphics pipeline, colours are always represented as RGBA values, either with _linear encoding_ or using an _sRGB transfer function_ applied. For many applications, the grapics API and the operating system will take care of converting these values to the correct colour space for the display (assuming that the display is correctly calibrated and that the operating system and GPU drivers support colour management with the correct colour profile loaded).

However, under some circumstances, you might want to perform colour management yourself. In this case, you need to provide `psychophysics` with the correct primaries and the correct transfer function for your display. If you intend to present colours that rely on the spectral properties of the display, you will also need to use a spectro-radiometer to measure the spectral power distribution of the display. With these information, `psychophysics` can convert all colours you provide to it to an appropriate RGB colour space and apply the correct transfer function. It will also try its best to convince your operating system and GPU to skip any further tone mapping and gamma correction.

> If you are not sure whether you need to perform colour management yourself, you probably don't need to. If you are not sure whether your display is correctly calibrated, you probably need to calibrate it.

For more information on colour management, see the section "Background on colour spaces" [below](#background-on-colour-spaces).

### Colour types

All colour handling in `psychophysics` is based on the `palette` crate (and this is also where new colour spaces should be added). The `palette` crate is a very powerful crate for handling colours and colour spaces. It provides a number of colour spaces and conversion functions between them.

`psychophysics` defines a number of colour types that are based on the `palette` crate. These types are:

- `SRGBA`: An RGBA colour value based on the sRGB primaries (+ alpha) with 32 bits of floating point precision per channel. Values are in the range [0.0, 1.0] and are expected to be encoded using the sRGB (piecewise power function) transfer function. The `LinearSRGBA` type is the same as `SRGBA`, but with a linear transfer function.
- `XYZA`: An XYZ (+ alpha) colour as defined by the CIE 1931 standard observer with 32 bits of floating point precision per channel.
- `YxyA`: A Yxy (+ alpha) colour with 32 bits of floating point precision per channel. This is a transformation of the XYZ colour space to a more perceptually uniform space.
- `DisplayP3RGB`: An RGBA colour in the Display P3 colour space with 32 bits of floating point precision per channel. Values are in the range [0.0, 1.0] and are expected to be encoded using the sRGB transfer function.
- `LMSA`: An LMS (+ alpha) colour with 32 bits of floating point precision per channel. This is a colour space that is based on the response of the three types of cones in the human retina.
- `DKLA`: A DKL (Derrington-Krauskopf-Lennie) (+ alpha) colour with 32 bits of floating point precision per channel. This is a colour space that is based on the response of the three types of cones in the human retina.

Most colour types can be converted to and from each other using the `IntoColour` trait provided by the `palette` crate. This means that you can easily convert a colour from one colour space to another. Note that this is not a lossless operation because different colour spaces capture different gamuts and then converting a colour from one colour space to another and back will not necessarily give you the same colour.

> Converting from and to `LMSA` and `DKLA` is a bit more complicated because these colour spaces are not based on the same principles as the other colour spaces. You will nedd to provide a conversion matrix (based on the spectral properties of the display and the observer) to convert between these colour spaces and the other colour spaces. This also means that you very likely want to disable any colour management that your operating system or GPU might be performing.

<div class="warning">
Once colour data is sent of the GPU, all further blending operations are performed 
in linear (RGB) space.

This is usually the correct thing to do, but it is important to be aware of this
when working with more exotic colour spaces like LMS or DKL or raw spectral data.

For these colour spaces, blending cannot be performed on the GPU and must be done before
sending the data to the graphics pipeline.
</div>

### Specifying colours

As discussed above, `psychophysics` requires you to specify the colour space when creating a colour. This is done by using the appropriate colour type. For example, if you would like to create a 50% gray colour in sRGB, you can do so as follows:

```
let grey = SRGBA::new(0.5, 0.5, 0.5, 1.0);
```

Note that this is not (!) the same as the following:

```
let grey = LinearSRGBA::new(0.5, 0.5, 0.5, 1.0);
```

The first example creates a 50% gray colour in sRGB colour space. The second example creates a 50% gray colour in linear sRGB colour space. The difference between these two colour spaces is that the sRGB colour space uses a non-linear transfer function to encode the colour values. In fact, 50% gray in the (non-linear) sRGB colour space is the same as approximately 21% gray in the linear sRGB colour space.

### Converting between colour spaces

The `palette` crate provides the `IntoColour` trait that allows you to convert between colour spaces. For example, if you would like to convert the 50% gray colour in sRGB to the linear sRGB colour space, you can do so as follows:

```
let grey = SRGBA::new(0.5, 0.5, 0.5, 1.0);
let grey_linear = grey.into_colour::<LinearSRGBA>();
```

**Note:** Converting between colour spaces is not lossless. This means that converting a colour from one colour space to another and back will not necessarily give you the same colour.
