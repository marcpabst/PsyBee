# Units

Most functions in psychopysics accept both numerical scalar values (such as `int` or `float`) and the `Size` type. Size types are used to represent physical quantities in psychophysics, and are used to ensure that the units of the quantities are consistent. For example, the `Size` type can be used to represent the size of a stimulus in degrees of visual angle.

You can define a `Size` value either by calling one of the constructors:

```python
from psychophysics.size import px, deg

size1 = px(100) # 100 pixels
size2 = deg(1)  # 1 degree of visual angle
```

Or, using shorthand syntax:

```python
from psychophysics.size import px, deg

size1 = 100 * px # 100 pixels
size2 = 1 * deg  # 1 degree of visual angle
```

You can add and subtract `Size` values, even if they have different units:

```python
size3 = size1 + size2 # 100 pixels + 1 degree of visual angle
```

You can also multiply and divide `Size` values by scalar values:

```python
size4 = size1 * 2 # 200 pixels
size5 = size2 / 2 # 0.5 degrees of visual angle
```



The following units are currently supported:

- `px`: pixels
- `pt`: points
- `sw`: fraction of screen width
- `sh`: fraction of screen height
- `deg`: degrees of visual angle
- `cm`: centimeters
- `mm`: millimeters
- `m` : meters
- `inch`: inches