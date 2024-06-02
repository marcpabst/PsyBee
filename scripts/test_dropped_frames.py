"""A simple example experiment using the psybee library."""

import logging
import os
import time

from psybee import MainLoop, ExperimentManager
from psybee.audio import AudioDevice, FileStimulus
from psybee.geometry import Circle, Rectangle, Pixels, ScreenWidth
from psybee.events import EventKind
from psybee.stimuli import GaborStimulus, ImageStimulus, SpriteStimulus
from psybee.window import Window


def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    window = exp_manager.create_default_window()

    (width, height) = (window.width_px, window.height_px)

    while True:
        frame = window.get_frame()

        window.present(frame)


if __name__ == "__main__":
    # Set the logging level
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"

    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.INFO)

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
