from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus, TextStimulus
from psydk.visual.color import rgb, linrgb
import sys
import numpy as np
# import tobii_eye_tracker
import time


def my_experiment(exp_manager) -> None:
    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    # tracker = tobii_eye_tracker.TobiiEyeTracker()
    # tracker.initialize()

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)

    checkerboard_sizes = [2,4,8,16,32,64,128,256,512]
    michelson_contrast = 1.0

    start_text = TextStimulus(
        "Press = or - to change luminance",
        font_size = 20,
        font_weight = "bold",
        y = -100,
        fill_color = linrgb(1, 1, 1, 1))

    lum_text = TextStimulus(
        "Luminance: ?",
        font_size = 20,
        font_weight = "bold",
        y = 100,
        fill_color = linrgb(1, 1, 1, 1))


    background = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "uniform")

    checkerboard = PatternStimulus(
        Shape.rectangle(width = "1vw", height = "1vh"),
        x = "-0.5vw", y = "-0.5vh",
        pattern = "checkerboard",
        fill_color = linrgb(1,0,0),
        background_color = linrgb(0,0,0),
        alpha = 1.0)


    fixation = ShapeStimulus(Shape.circle(10), fill_color=(1, 1, 1, 1))

    checkerboard_size_index = 0

    global luminance
    luminance = 0.740

    def lum_change(e):
        global luminance
        if e.key == "=":
            luminance = min(luminance + 0.001, 1)
        elif e.key == "-":
            luminance = max(luminance - 0.001, 0)

    main_window.add_event_handler("key_press", lum_change)

    start_text.animate("font_size", 10000, 10.0)

    for i in range(10000000):
        frame = main_window.get_frame()

        # gaze_points = tracker.get_gaze_points()
        # for gaze_point in gaze_points:
        #     print(gaze_point)
        #     fixation["x"] = (gaze_point[0] - 0.5) * 1920
        #     fixation["y"] = (gaze_point[1] - 0.5) * 1080



        # set background luminance
        background["fill_color"] = linrgb(luminance, 0, 0)
        lum_text["text"] = f"Luminance: {luminance:.3f}"

        frame.draw(background)

        if i % 10 > 5:
            # checkerboard["phase_x"] = (checkerboard["phase_x"] + 180) % 360
            frame.draw(checkerboard)



        if i % 100 == 0:
            checkerboard["cycle_length"] = checkerboard_sizes[checkerboard_size_index]
            checkerboard_size_index = (checkerboard_size_index + 1) % len(checkerboard_sizes)


        frame.draw(start_text)
        frame.draw(lum_text)
        frame.draw(fixation)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
