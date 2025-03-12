from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
from psydk.visual.color import rgb, linrgb
import sys
import numpy as np



def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=2)

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)

    checkerboard_sizes = [4,8,40,100]
    michelson_contrast = 1.0

    background = PatternStimulus(
        Shape.rectangle(width = "0.5vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "uniform",
        fill_color = linrgb(0.50, 0.50, 0.50)
    )

    stimulus1 = PatternStimulus(
        Shape.rectangle(width = "0.5vw", height = "1vh"),
        x = 0, y = "-0.5vh",
        pattern = "uniform",
        fill_color = linrgb(0.3,0.3,0.3)
    )

    stimulus2 = PatternStimulus(
        Shape.rectangle(width = "0.5vw", height = "1vh"),
        x = 0, y = "-0.5vh",
        pattern = "uniform",
        fill_color = linrgb(0.7,0.7,0.7)
    )

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

        if i % 2 == 0:
            frame.draw(stimulus1)
        else:
            frame.draw(stimulus2)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
