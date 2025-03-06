from psybee import run_experiment
from psybee.visual.geometry import Transformation2D, Shape
from psybee.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
from psybee.visual.color import rgb, linrgb
import sys
import numpy as np



def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=2)

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)

    checkerboard_sizes = [4]
    michelson_contrast = 1.0

    background = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "uniform",
    )

    checkerboard = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "checkerboard",
        fill_color = linrgb(0.25, 0.25, 0.25),
        background_color = linrgb(0.75, 0.75, 0.75),
        alpha = 1.0,
    )


    fixation = ShapeStimulus(Shape.circle(10), fill_color=(1, 0, 0, 1))

    checkerboard_size_index = 0

    global luminance
    luminance = 0.2198
    luminance = 0.5

    def lum_change(e):
        global luminance
        if e.key == "=":
            luminance = min(luminance + 0.001, 1)
        elif e.key == "-":
            luminance = max(luminance - 0.001, 0)

    main_window.add_event_handler("key_press", lum_change)

    for i in range(10000000):
        frame = main_window.get_frame()

        # set background luminance
        background["fill_color"] = linrgb(luminance, luminance, luminance)

        frame.draw(background)

        if i % 200 > 100:
            # checkerboard["phase_x"] = (checkerboard["phase_x"] + 180) % 360
            frame.draw(checkerboard)
            frame.draw(fixation)

        if i % 500 == 0:
            checkerboard["cycle_length"] = checkerboard_sizes[checkerboard_size_index]
            checkerboard_size_index = (checkerboard_size_index + 1) % len(checkerboard_sizes)



        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
