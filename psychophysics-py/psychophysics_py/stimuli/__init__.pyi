# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class Stimulus:
    """
    Represents a Stimulus. This class is used either as a base class for other
    stimulus classes or as a standalone class, when no specific runtume type
    information is available.
    """

    def contains(self, x, y):
        """ """
        pass

    def hide(self):
        """ """
        pass

    def show(self):
        """ """
        pass

    def toggle_visibility(self):
        """ """
        pass

    @property
    def visible(self):
        """ """
        pass

class GaborStimulus(Stimulus):
    """
    A GaborStimulus.

    Consists of a Gabor patch, which is a sinusoidal grating enveloped by a
    Gaussian envelope.

    Parameters
    ----------
    window : Window
        The window that the stimulus will be presented on.
    shape : Shape
        The shape of the stimulus.
    phase : float
        The phase of the sinusoidal grating in radians.
    cycle_length : Size
        The length of a single cycle of the sinusoidal grating.
    std_x : Size
        The standard deviation of the Gaussian envelope in the x direction.
    std_y : Size
        The standard deviation of the Gaussian envelope in the y direction in pixels.
    orientation : float
        The orientation of the sinusoidal grating in adians.
    color : tuple
     The color of the stimulus as an RGB tuple.

    Returns
    -------
    GaborStimulus :
     The GaborStimulus that was created.
    """

    def color(self):
        """ """
        pass

    def contains(self, x, y):
        """ """
        pass

    def cycle_length(self):
        """ """
        pass

    def hide(self):
        """ """
        pass

    def orientation(self):
        """ """
        pass

    def phase(self):
        """ """
        pass

    def set_color(self, color):
        """ """
        pass

    def set_cycle_length(self, cycle_length):
        """ """
        pass

    def set_orientation(self, orientation):
        """ """
        pass

    def set_phase(self, phase):
        """ """
        pass

    def set_translation(self, x, y):
        """ """
        pass

    def show(self):
        """ """
        pass

    def toggle_visibility(self):
        """ """
        pass

    def translate(self, x, y):
        """ """
        pass

    @property
    def visible(self):
        """ """
        pass

class ImageStimulus(Stimulus):
    """
    An ImageStimulus.
    """

    def contains(self, x, y):
        """ """
        pass

    def hide(self):
        """ """
        pass

    def set_translation(self, x, y):
        """ """
        pass

    def show(self):
        """ """
        pass

    def toggle_visibility(self):
        """ """
        pass

    @property
    def visible(self):
        """ """
        pass

class SpriteStimulus(Stimulus):
    """
    A SpriteStimulus that can be used to display an animated sprite.
    """

    def advance_image_index(self):
        """ """
        pass

    def contains(self, x, y):
        """ """
        pass

    def hide(self):
        """ """
        pass

    def reset(self):
        """ """
        pass

    def set_translation(self, x, y):
        """ """
        pass

    def show(self):
        """ """
        pass

    def toggle_visibility(self):
        """ """
        pass

    @property
    def visible(self):
        """ """
        pass

class VideoStimulus(Stimulus):
    """
    A VideoStimulus.
    """

    def contains(self, x, y):
        """ """
        pass

    def hide(self):
        """ """
        pass

    def init(self):
        """ """
        pass

    def pause(self):
        """ """
        pass

    def play(self):
        """ """
        pass

    def show(self):
        """ """
        pass

    def toggle_visibility(self):
        """ """
        pass

    @property
    def visible(self):
        """ """
        pass
