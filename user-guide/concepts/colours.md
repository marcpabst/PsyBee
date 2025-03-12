# Colours
<div class="grid cards" markdown>
- :fontawesome-solid-quote-right: _The rays, to speak properly, are not coloured. In them there is nothing else than a certain power and disposition to stir up a sensation of this or that colour._
<sub>**Isaac Newton (1704), Opticks**</sub>
</div>


Correct handling of colours is important for investigating many interesting questions in cognitive and perceptual science. Unfortunately, colours and colour spaces are complex and can be difficult to understand and work with. This document is intended to provide a brief overview over the handling of colours in the *psydk* library.

Please read the background information on [colours and colour spaces](./colours-and-colour-spaces.md) if you are interested in a more detailed discussion of the topic.

Key points:

- __There is currently no dedicated support for full colour managment in the *psydk* library.__
  As such, the library is limited to working with colours in the RGB colour space using your monitor's primaries. Crucially, this puts the responsibility on the user to ensure that colours are handled correctly.
- __However, the library provides a number of tools to help you work with colours in the RGB colour space.__
  This mainly concerns the RGB *encoding functions*, describing the non-linear relationship between the RGB values as defined in a particular RGB colour space and the actual light emitted by the monitor. On top of that, the library provides a way to correct for non-linearities in the monitor's gamma curve (that deviates from the standard sRGB gamma curve). Taken, together, these tools allow for what is often called "gamma correction".

## Gamma correction

Imagine your goal is to display four grey squares with different intensities on your monitor, with luminance values spaces equidistantly between 0 and 1. You might be tempted to simply set the RGB values of the squares to the corresponding luminance values. So, for example, the RGB values for the squares with luminance values of 0.25, 0.5, 0.75, and 1.0 would be `rgb(0.25, 0.25, 0.25)`, `rgb(0.5, 0.5, 0.5)`, `rgb(0.75, 0.75, 0.75)`, and `rgb(1.0, 1.0, 1.0)`, respectively. Once you're done, you might notice that the squares do actually appear to linearly increase in brightness. This is great, right?

Well, not quite. As you might know, human perception of brightness is not linear. So if stimuli *appear* to increase linearly in (perceivded) brightness, they are most likely not *actually* increasing linearly in (physical) luminance. The reason for this is that the relationship between the RGB values and the actual light emitted by the monitor is non-linear. This non-linear relationship is often referred to as the monitor's gamma curve or, more correctly, as the colour space's encoding function.

The process of compensating for the non-linear relationship between the RGB values and the actual light emitted by the monitor is usually called *gamma correction*.
