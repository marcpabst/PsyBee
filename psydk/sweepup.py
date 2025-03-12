from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
from psydk.visual.color import rgb, linrgb
import sys
import numpy as np



def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)


    background = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "uniform",
        fill_color = linrgb(0.50, 0.50, 0.50)
    )
    lum = 0
    for i in range(10000000):
        frame = main_window.get_frame()

        # update background
        lum = (i % 200) / 200
        background["fill_color"] = linrgb(lum, lum, lum)


        frame.draw(background)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
