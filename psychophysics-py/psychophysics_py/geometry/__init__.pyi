# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class Shape:
    pass

class Size:
    """
    A generic size.
    """

class Circle(Shape):
    """
    A Circle shape defined by a center (x, y) and a radius.
    """

class Pixels(Size):
    """
    A `Size` in pixels.

    Parameters
    ----------
    value : float
       The value of the size in pixels.
    """

class Rectangle(Shape):
    @staticmethod
    def fullscreen():
        """
        Create a fullscreen rectangle

        Returns
        -------
        Rectangle :
          The fullscreen rectangle that was created.
        """
        pass

class ScreenHeight(Size):
    pass

class ScreenWidth(Size):
    pass
