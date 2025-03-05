from psybee import run_experiment
from psybee.visual.geometry import Transformation2D, Shape
from psybee.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
from psybee.visual.color import rgb
import sys
import numpy as np



def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    checkerboard_sizes = [10, 20, 30, 100]

    checkerboard = PatternStimulus(
        Shape.rectangle(width = "0.8vw", height = "0.8vh"),
        x = "-0.4vw", y = "-0.4vh",
        pattern = "checkerboard",
        fill_color = rgb(0.0, 0.0, 0.0),
        background_color = rgb(1.0, 1.0, 1.0)
    )

    fixation = ShapeStimulus(Shape.circle(10), fill_color=(1, 0, 0, 1))

    checkerboard_size_index = 0

    for i in range(10000000):
        frame = main_window.get_frame()

        if i % 20 == 0:
            checkerboard["phase_x"] = (checkerboard["phase_x"] + 180) % 360

        frame.draw(checkerboard)

        if i % 200 == 0:
            checkerboard["cycle_length"] = checkerboard_sizes[checkerboard_size_index]
            checkerboard_size_index = (checkerboard_size_index + 1) % len(checkerboard_sizes)

        frame.draw(fixation)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
