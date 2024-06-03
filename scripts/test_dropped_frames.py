"""A simple example experiment using the psybee library."""

import logging
import os
import time

from psybee import MainLoop, ExperimentManager
from psybee.audio import AudioDevice, FileStimulus
from psybee.geometry import Circle, Rectangle, Pixels, ScreenWidth
from psybee.events import EventKind
from psybee.stimuli import GaborStimulus, ImageStimulus, SpriteStimulus, ColorStimulus
from psybee.window import Window


def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    window = exp_manager.create_default_window()

    (width, height) = (window.width_px, window.height_px)

    c1 = (233 / 255, 113 / 255, 113 / 255)
    c2 = (148 / 255, 253 / 255, 253 / 255)

    stim = ColorStimulus(window, Rectangle.fullscreen(), c1)

    window.stimuli.append(stim)

    for i in range(int(1e6)):
        frame = window.get_frame()

        # change the color of the stimulus
        if i % 2 == 0:
            stim.color = c1
        else:
            stim.color = c2

        #print(stim.color)

        window.present(frame)


if __name__ == "__main__":
    # Set the logging level
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"

    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.INFO)

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
