# Windows and Frames

## Window

A `Window` is an abstraction that represents the display screen on which visual stimuli are presented. It is the primary interface for creating and managing the visual output of a PsyBee experiment. The window provides a canvas on which you can draw visual stimuli, such as shapes, images, and text, and it handles the rendering of these stimuli to the screen. The window also provides methods for handling input events, such as keyboard, mouse and touchscreen interactions.

 While usually you will find it more useful to add stimuli to a `Frame` object, and then submit the frame to the window (see below), you can also add stimuli directly to the window. These stimuli will be automatically added to the any frame that is submitted to the window. This can be helpful when you don't need frame-by-frame control over the stimuli, or when you want to add stimuli that are present throughout longer periods of time. This also allows to add event handlers to individual stimuli, which provides an elegant way to handle events that are tied to specific stimuli (e.g. a button that can be hovered over and clicked).

!!! warning
    For technical reasons, all stimuli must be associated with a window. This is because logically, different windows can be associated with different graphic adapters, and stimuli need to be associated with the same graphic adapter as the window they are rendered on. If you try to add a stimulus to a window that is different from the window it was created on, an error will be raised.

### Creating a Window

When an experiment is started, an `ExperimentManager` object is passed to the experiment function as the sole argument. The `ExperimentManager` object provides a method `create_default_window` that can be used to create a window with default settings.

```python
from psybee import run_experiment

def my_experiment(experiment_manager):
    window = experiment_manager.create_default_window()
    # Add stimuli and event handlers to the window
    # ...
    window.close()

run_experiment(my_experiment)
```

### Window Settings

In most cases, you will want to customize the settings of the window to match the requirements of your experiment. You can do this by passing a `WindowOptions` object to the `create_window` method. The `WindowOptions` object allows you to specify various settings for the window, such as the screen resolution, refresh rate, and whether the window should be fullscreen or windowed. The `ExperimentManager` will then do its best to create a window that matches the specified settings (or else raise an error if it is not possible).

The following window options are available:

- `Windowed`: Create a windowed window with a specific resolution (or default resolution if none is specified) - only supported on desktop platforms.
- `FullscreenExact`: Create a fullscreen window with a specific resolution and refresh rate (or default values if none are specified) on a specific monitor (or the primary monitor if none is specified). If you specify a resolution or refresh rate that is not supported by the monitor, an error will be raised.
- `FullscreenHighestRefreshRate`: Create a fullscreen window with the highest refresh rate that is supported by the monitor and matches the specified resolution (or default resolution if none is specified) on a specific monitor (or the primary monitor if none is specified). If you specify a resolution that is not supported by the monitor, an error will be raised.
- `FullscreenHighestResolution`: Create a fullscreen window with the highest resolution that is supported by the monitor and matches the specified refresh rate (or default refresh rate if none is specified) on a specific monitor (or the primary monitor if none is specified). If you specify a refresh rate that is not supported by the monitor, an error will be raised.

```python
from psybee import run_experiment, window_options

def my_experiment(exp_manager):
    win_opts = window_options.FullscreenExact(
        resolution=(1920, 1080), 
        refresh_rate=60)
    window = exp_manager.create_window(win_opts)

    # Do stuff with the window

    window.close()

run_experiment(my_experiment)
```

## Frame

A `Frame` refers to a single image within a sequence of images that collectively form the visual output on a display. It is a fundamental unit that represents the entire scene at a specific instance, encapsulating all graphical elements such as objects, textures, colors, and lighting. On a fundamental level, frames are rendered by the graphics processor which turns thw abstract descriptions of what is supposed to be shown on screen into pixels arranged in a grid pattern, each pixel containing color information that contributes to the overall image.

In the context of PsyBee, a `Frame` holds the visual stimuli that are presented on the screen at a specific point in time. To show any stimuli on the screen, you need to first obtain a `Frame` object, add the stimuli to it, and then `submit` the frame back to the window. The window will then display the frame on the screen. It depends on the window's refresh rate how often you will need to do this, but typically, you will need to submit a new frame every 16.67 milliseconds (for a 60 Hz display).