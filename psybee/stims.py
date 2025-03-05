from psybee import run_experiment
from psybee.visual.geometry import Transformation2D, Shape
from psybee.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
from psybee.visual.color import rgb, linrgb
import sys
import numpy as np



def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    checkerboard_sizes = [200, 80, 40, 8]

    background = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "uniform",
        fill_color = linrgb(0.52, 0.52, 0.52)
    )

    checkerboard = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "checkerboard",
        fill_color = linrgb(0.2, 0.2, 0.2),
        background_color = linrgb(0.8, 0.8, 0.8)
    )

    fixation = ShapeStimulus(Shape.circle(10), fill_color=(1, 0, 0, 1))

    checkerboard_size_index = 0

    global luminance
    luminance = 0.445


    def lum_change(e):
        global luminance
        if e.key == "=":
            luminance = min(luminance + 0.005, 1)
        elif e.key == "-":
            luminance = max(luminance - 0.005, 0)

    main_window.add_event_handler("key_press", lum_change)

    for i in range(10000000):
        frame = main_window.get_frame()

        # set background luminance
        background["fill_color"] = linrgb(luminance, luminance, luminance)

        frame.draw(background)

        if i % 30 < 15:
            # checkerboard["phase_x"] = (checkerboard["phase_x"] + 180) % 360
            frame.draw(checkerboard)

        if i % 200 == 0:
            checkerboard["cycle_length"] = checkerboard_sizes[checkerboard_size_index]
            checkerboard_size_index = (checkerboard_size_index + 1) % len(checkerboard_sizes)

        frame.draw(fixation)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
