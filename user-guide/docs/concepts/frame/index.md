# Windows and Frames

## Window

A `Window` is an abstraction that represents the display screen on which visual stimuli are presented. It is the primary interface for creating and managing the visual output of a psychophysics experiment. The window provides a canvas on which you can draw visual stimuli, such as shapes, images, and text, and it handles the rendering of these stimuli to the screen. The window also provides methods for handling input events, such as keyboard, mouse and touchscreen interactions.

 While usually you will find it more useful to add stimuli to a `Frame` object, and then submit the frame to the window (see below), you can also add stimuli directly to the window. These stimuli will be automatically added to the any frame that is submitted to the window. This can be helpful when you don't need frame-by-frame control over the stimuli, or when you want to add stimuli that are present throughout longer periods of time. This also allows to add event handlers to individual stimuli, which provides an elegant way to handle events that are tied to specific stimuli (e.g. a button that can be hovered over and clicked).

!!! warning
    For technical reasons, all stimuli must be associated with a window. This is because logically, different windows can be associated with different graphic adapters, and stimuli need to be associated with the same graphic adapter as the window they are rendered on. If you try to add a stimulus to a window that is different from the window it was created on, an error will be raised.

## Frame

A `Frame` refers to a single image within a sequence of images that collectively form the visual output on a display. It is a fundamental unit that represents the entire scene at a specific instance, encapsulating all graphical elements such as objects, textures, colors, and lighting. On a fundamental level, frames are rendered by the graphics processor which turns thw abstract descriptions of what is supposed to be shown on screen into pixels arranged in a grid pattern, each pixel containing color information that contributes to the overall image.

In the context of psychophysics, a `Frame` holds the visual stimuli that are presented on the screen at a specific point in time. To show any stimuli on the screen, you need to first obtain a `Frame` object, add the stimuli to it, and then `submit` the frame back to the window. The window will then display the frame on the screen. It depends on the window's refresh rate how often you will need to do this, but typically, you will need to submit a new frame every 16.67 milliseconds (for a 60 Hz display).