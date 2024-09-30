#!/usr/bin/env python

import time
import pathlib
import numpy as np
from psybee import ExperimentManager, WindowOptions, MainLoop, GaborStimulus, Transformation2D, GaborStimulus, TextStimulus, Rgba


def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    print("Creating window")
    window = exp_manager.create_default_window()
    print("Window created")

    gabor = GaborStimulus(
        0, 0,  # center x, y
        "5cm", # size
        ".5cm",# cycle length
        "5cm", # sigma
    )

    gabor.hide()

    start_text = TextStimulus(0,0   ,
        "Click to start",
        "1cm",
        fill = Rgba(1, 1, 0, 1.0))


    start_text = start_text.rotated_at(0, "1cm", "3cm").scaled_at(0.5, 0.5, 0, 0)

    print(start_text)

    def mouse_move_handler(event):
        gabor["cx"] = event.position[0]
        gabor["cy"] = event.position[1]

        start_text["x"] = event.position[0]
        start_text["y"] = event.position[1]

    def mouse_click_handler(event):
        start_text.hide()
        gabor.show()

    # add event handlers
    window.add_event_handler("CursorMoved", mouse_move_handler)
    window.add_event_handler("MouseButtonPress", mouse_click_handler)

    # main loop
    for i in range(int(1e6)):
        frame = window.get_frame() # get a frame

        # phase reversal
        frame.add(gabor)
        frame.add(start_text)

        window.present(frame)

if __name__ == "__main__":
    print("Running experiment")
    # Run the experiment
    MainLoop().run_experiment(my_experiment)
