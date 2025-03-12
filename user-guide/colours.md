# Colours and Gamma Correction

___This article proides background information on colour spaces and gamma correction. *psydk* currently does not provide full colour management support, but you can specify the encoding used and you can perform gamma calibration yourself.___

A colour space is a mathematical model that describes how colours can be represented as tuples of numbers. Since human vision is trichromatic (i.e. we have three types of colour receptors in our eyes), most colour spaces are tristimulus colour spaces, meaning that colours can be represented as a combination of three numbers.


## Useful colour spaces

### RGB colour spaces

RGB colour spaces are tristimulus colour spaces that are based on defining values for red, green, and blue (so-called "primaries") and a white point. They are the most ubiquitous colour spaces in computer graphics and will probably be the most familiar to you.

!!! Terminology
    It can be slightly confusing that the term "RGB colour space" is used to refer to both the concept of a colour space represented by red, green, and blue, and to a specific colour space that is defined by a set of specific red, green, and blue primaries and a white point (like sRGB or Adobe RGB). **For sake of clarity, you should always clearly state which RGB colour space you are referring to.**

The three most important concepts in RGB colour spaces are: the **primaries**, the **white point**, and the **encoding**.

The **primaries** are the three colours that are used to represent all other colours. Any given, clearly defined colour space will have a distinct set of primaries. For example, the sRGB colour space uses the following primaries (in CIE xyY colour space):

- Red: (X = 0.64, Y = 0.33)
- Green: (X = 0.30, Y = 0.60)
- Blue: (X = 0.15, Y = 0.06)

The **white point** is the colour of a perfectly white surface, i.e. when all three primary colours are present in equal amounts. White points are usually defined in terms of their spectral power distribution, and the most commonly used white point is the CIE standard illuminant D65, which is characterized by a colour temperature of approximately 6500 K [^3] [^4].

The **encoding** is the transfer function that is used to convert the RGB values to light intensity (also called electro-optical transfer function, see [below](#converting-numbers-to-light-intensity)). The encoding can be linear or non-linear.


Confusingly, some definitions of certain RGB colour spaces include the encoding, while others do not. For example, the sRGB colour space is usually used with with so-called "sRGB transfer function" (a non-linear encoding), but you sometimes the sRGB primaries and white point are used with a linear encoding (often called "linear sRGB"). Even more confusingly, not all implementations of sRGB use the same transfer function. The sRGB transfer function is formally defined by an ICE standard, and it is a piecewise function that combines a linear segment with a power function with an exponent of 2.4\. However, some implementations of sRGB use a slightly different transfer function. For example, many displays approximate the sRGB transfer function using a power function with an exponent of 2.2\. Annoyingly, some (even professional) software packages have needlessly invented their own transfer functions for sRGB, which can cause confusion and compatibility issues.[^5]

RGB colour spaces are extremely useful for computer graphics because they are very intuitive and easy to understand. Moreover, almost all modern display technologies are based on emitting red, green, and blue light, which makes RGB colour spaces a natural choice for representing colours that are to be displayed on a screen. Some common RGB colour spaces are:

- sRGB (the standard colour space for most consumer applications)
- Adobe RGB (a colour space that is used in professional photography and printing)
- Display P3 (a colour space that is used in Apple's devices)
- Rec. 709 (a colour space that is used in HDTV systems)
- Rec. 2020 (a colour space that is used in UHDTV systems)

### CIE 1931 XYZ colour space

The CIE XYZ colour space captures all perceivable colours through three components: `X`, `Y`, and `Z`. Where `Y` denotes luminance, while `X` and `Z` denote colour information. Because it's device-independant, it is often used a a universal standard for colour representation. Also, the CIE XYZ colour space is the foundation for many other colour spaces, including the CIE xyY colour space, which is a transformation of XYZ to a more perceptually uniform space.

### LMS colour space

The LMS colour space is a colour space that is based on the response of the three types of cones in the human retina. It is a linear colour space, meaning that the values are proportional to the number of photons that are absorbed by the cones. This makes it a very useful colour space for vision scientists, because it allows us to represent colours in a way that is directly related to the physical properties of the light that is absorbed by the cones. However, it is not a very intuitive colour space for most people, and because it is linke to the spectral properties of both the display and the observer, it can be difficult to work with in practice.

## Converting numbers to light intensity

How do we go from three colour values to light intensity? This is where the concept of the electro-optical transfer function (EOTF) comes in.

The electro-optical transfer function (EOTF) is a function that describes how the RGB values are converted to light intensity. In other words, it describes how the RGB values are converted to actual number of photons emitted by the display. In the simplest case, the EOTF is a linear function that simply scales the RGB values by a constant factor. For example, this would mean that an RGB value of `(100, 100, 100)` would result in twice as much light being emitted as an RGB value of `(50, 50, 50)`.

However, most displays use a non-linear EOTF. This is for both historical, perceptual, and technical reasons.

> The following section contains some semi-techincal and vaguely interesting information about the history of display technology. Feel free to skip it if you are not interested in this.

Until the late 1990s, most displays were based on the cathode ray tube (CRT) technology. In CRT displays, an electron beam is used to excite phosphors on the screen, which then emit light. The brightness of the light emitted is a function of the voltage difference between the electron gun (cathode) and the grid (anode). This function is, as it turns out, highly non-linear (it is a somewhat common misconception that the nonlinearity of a CRT is due to the phosphor, but this is not the case; in fact, the nonlinearity is due to the physics of the electron beam while the phosphor response is actually quite linear) [^1].

Here is really interesting part: If you are fammiliar with basic human physiology, you might be aware that the human visual system also behaves non-linearly (in fact, this is a principle that holds for most sensory systems and was first described by Gustav Theodor Fechner and Ernst Heinrich Weber in the 19th century).

This means that the perceived brightness of a light is not proportional to the actual number of photons emitted, but roughly follows a power law. This is an amazing coincidence, because it means that the non-linear response of the human visual system is almost perfectly matched to the _inverse_ of the non-linear response of the CRT.

Why is this important?

When encoding colour values in a limited number of bits per channel (e.g. 8 bits per channel or 256 values per channel as is common in computer graphics), it would be a bad idea to spread the values uniformly across the entire range.

> **Example: Linear encoding**

> Assuming 256 values per channel, a linear encoding would mean that the difference between the darkest and the brightest value is 255\. This would mean that the difference between the darkest and the second darkest value is 1/255\. This might sound like a small difference, but it is actually a huge difference in terms of perceived light intensity. In fact, the ratio of the perceived intensities of the darkest and the second darkest value is over 6%, well above the threshold of visibility.

> On the other hand, the difference between the brightest and the second brightest value is also 1/255, but this is a much smaller difference in terms of perceived light intensity. In fact, the ratio of the perceived intensities of the brightest and the second brightest value is less than 0.3%. That is completely invisible to the human eye!

but rather concentrate them where the human visual system is most sensitive. In practive, one shoud use more of the 256 values to represent darker shades, and fewer to represent brighter shades. Mathematically, this mweans that we should use an _encoding_ function that is the inverse of the human visual system's _decoding_ function.

The fact that the CRT's non-linear response is almost perfectly matched to the human visual system's non-linear response is a happy coincidence, and it means that it was possible to exploit the CRT's non-linear response to encode the colour values in a way that is more efficient than a linear encoding.

Fast forward to today, and we no longer use CRT displays. However, the non-linear EOTF has stuck around and has been adopted as the standard for most modern displays. This is because it is still a very efficient way to encode colour values, and because it is compatible with the vast amount of content that has been created for and using CRT displays.

## sRGB colour space

The most commonly used EOTF today is the sRGB transfer function.

The sRGB color space, which is the standard color space for numerous consumer applications, was developed by HP and Microsoft in 1996 ans was later regosnised by the International Electrotechnical Commission (IEC) [^2]. sRGB incorporates the Rec. 709 primaries, which were originally introduced in 1990 for HDTV systems under the ITU-R Recommendation BT.709\. The white point in sRGB is the CIE standard illuminant D65, characterized by a color temperature of approximately 6500 K[^3].

The transfer function of sRGB is non-linear and piecewise linear, approximating a gamma of 2.2:

```rust
if c <= 0.0031308
   c * 12.92
else
  1.055 * c^(1.0 / 2.4) - 0.055
```

where `c` is the RGB in the range [0.0, 1.0].

As described above, this encoding method was initially tailored for the gamma characteristics of CRT displays. Coincidentally, it also mimics the response of the human visual system to light in daylight conditions, making linearly spaced RGB values correspond to perceived linear light intensity.

## Gamma correction

!!! Terminology
    Note that the term "gamma correction" might be used slightly differently in other contexts.

When vision scientists talk about "gamma correction", they are usually referring to the process of compensating for the non-linear transfer function of the display. The goal of gamma correction in this context is to ensure that the light intensity emitted by the display is proportional to the RGB values that are sent to the display. This is important because it means that the RGB values can be interpreted as light intensity.

It is important to note that all modern graphics libraries and hardware perform some sort of gamma correction automatically because it is a fundamental part of the display pipeline to enure correct colour and alpha blending. However, by default, values are then automatically re-encoded into a (usually non-linear) colour space before being sent to the display.

If your goal is to present stimuli with a specific light intensity, you have a number of options to deal with this:

1. **You either trust the display manufacturer to have done a good job at calibrating the display or you calibrate the display yourself, using the tools privded by your operating system or the display manufacturer.** You then provide the correct colour values in the correct (linear!) colour space, and let the rendering pipeline take care of the rest. This means basically telling your GPU to skip the conversion into linear space and then have the linear values correctly encoded into the display's colour space. This is the most common approach when working colour managed applications. However, it requires that you have a well-calibrated display and that your operating system, graphics API, and GPU drivers all correctly support colour management.

2. You can perform the gamma correction yourself, either by transforming colours before you pass them to the rendering pipeline, by setting a LUT in the GPU set through your operating system, or by correcting it yourself before the framebuffer is sent to the display (either using a transfer function in a fragment shader or by using a LUT in the GPU).

*psydk* allows you to combine these two approaches. By default, your operating system and GPU drivers will take care of the gamma correction, but you can also _correct for innacuracies_ in the display's gamma correction by performing the gamma correction yourself. This basically means that we provide a mapping from the theoretically expected light intensity to the actual light intensity emitted by the display. **Note that this approach is slightly different from the traditional approach to gamma correction, which is to provide a mapping from whatever colour space you are using to the actual light intensity emitted by the display.**

*psydk* provides a number of tools to help you calibrate your display, and to ensure that the gamma correction is performed correctly.

[^1]: Charles Poynton, A Technical Introduction to Digital Video. New York: Wiley, 1996.

[^2]: The sRGB colour space is defined by the International Electrotechnical Commission (IEC) in the standard IEC 61966-2-1:1999.

[^3]: Following the re-definition of several physical constants in 1968 by the International Committee for Weights and Measures (CIPM), there was a minor shift in the Planckian locus. As a result, the CIE standard illuminant D65 is not precisely at 6500 K, but rather at 6504 K.

[^4]: If you are wondering why this white point was chosen, it apparently matches the colour of normal daylight in western/central Europe.

[^5]: Will the Real sRGB Profile Please Stand Up?, <https://ninedegreesbelow.com/photography/srgb-profile-comparison.html>
