# Sizes

<div class="grid cards" markdown>
- :fontawesome-solid-quote-right: _When you can measure what you are speaking about, and express it in numbers, you know something about it; but when you cannot measure it, when you cannot express it in numbers, your knowledge is of a meagre and unsatisfactory kind._  
<sub>**Lord Kelvin (1824-1907), in a lecture to the Institution of Civil Engineers**</sub>
</div>


Most functions in psychopysics accept both numerical scalar values (such as `int` or `float` - these will **always** be interpreted as pixels) and the `Size` type. Size types are used to represent physical quantities in PsyBee, and are used to ensure that the units of the quantities are consistent. For example, the `Size` type can be used to represent the size of a stimulus in degrees of visual angle.

Once you have brought the `PsyBee.size` module into your namespace, you can create `Size` values either by calling the constructor functions, ot by using the shorthand syntax. For example, you can create a `Size` value representing 100 pixels like this:

```python
from psybee.size import px

width = px(100) # using the constructor function, or
width = 100*px # using the shorthand syntax
```

The following units are currently supported:

- `px`: pixels
- `pt`: points
- `deg`: degrees of visual angle
- `sw`: fraction of screen width
- `sh`: fraction of screen height
- `deg`: degrees of visual angle
- `cm`: centimeters
- `mm`: millimeters
- `m` : meters
- `inch`: inches

!!! info
    Note that for correct results, you will need to set the correct physical size of the screen and the viewing distance. This will ensure that physical units like centimeters or degrees of visual angle are correctly converted to pixels for rendering.

## Size arithmetic

You can add and subtract `Size` values, even if they not have the same units. You can also multiply and divide `Size` values by scalar values. For example:

```python
from psybee.size import px, cm, sw

width = 1*sw # full screen width
left_margin = 2*cm # 2 centimeters

new_width = width - 2*left_margin # full screen width - margins
```

!!! warning
    You cannot multiply or divide `Size` values by other `Size` values. This is because the result of such an operation would change the dimension of the value, so that the result would no longer represent a size. For example, multiplying two pixel values would result in a *pixel squared* value, which is a measure of area, not a size. 

## Converting between units

You can convert `Size` values between units using the `to` method. You can then extract the numerical value of the converted size using the `value` attribute. For example:

```python
from psybee.size import px, cm

width = 100*px
width_cm = width.to(cm) # convert to centimeters
width_cm.value # obtain the numerical value
```

