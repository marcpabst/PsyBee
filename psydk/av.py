from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus, TextStimulus, Stimulus
from psydk.visual.color import rgb, linrgb
import sys
import numpy as np
import time
import pandas as pd

exp_params = {
    "fps": 120,
    "trial_duration_ms": 150,
    "fixation_duration_ms": 1000,
    "probability_rare": 0.1,
    "n_trials": 20,
    "background_color": linrgb(0.50, 0.50, 0.50)
}


class Instructions():
    def __init__(self):

        self.text0 = TextStimulus(
            "Press button when you see a RARE target",
            font_size = 130,
            fill_color = linrgb(0, 0, 0, 1),
            y = "6deg")

        self.target1 = PatternStimulus(
            Shape.circle(radius = "3deg"),
            x = "-5deg", y = "0deg",
            pattern = "stripes",
            pattern_rotation = 0,
            stroke_color = linrgb(1, 0, 0),
            stroke_width = 15,
            fill_color = linrgb(0.50, 0.50, 0.50))

        self.text1 = TextStimulus(
            "Do nothing",
            font_size = 100,
            fill_color = linrgb(0, 0, 0, 1),
            x = "-5deg", y = "-4deg")

        self.target2 = PatternStimulus(
            Shape.circle(radius = "3deg"),
            x = "5deg", y = "0deg",
            pattern = "stripes",
            pattern_rotation = 90,
            stroke_color = linrgb(0, 1, 0),
            stroke_width = 15,
            fill_color = linrgb(0.50, 0.50, 0.50))


        self.text2 = TextStimulus(
            "Press button",
            font_size = 100,
            fill_color = linrgb(0, 0, 0, 1),
            x = "5deg", y = "-4deg")

    def draw(self, frame):
        frame.draw(self.text0)
        frame.draw(self.target1)
        frame.draw(self.text1)
        frame.draw(self.target2)
        frame.draw(self.text2)


def my_experiment(exp_manager) -> None:
    # # crreate the main data frame
    # trials = pd.DataFrame(columns = ["trial", "stimulus", "response", "rt", "correct"])

    # # add the trials
    # for i in range(n_trials):
    #     #


    # create a new window
    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)

    trial_duration_frames = int(exp_params["fps"] * exp_params["trial_duration_ms"] / 1000)
    fixation_duration_frames = int(exp_params["fps"] * exp_params["fixation_duration_ms"] / 1000)

    fixation = PatternStimulus(
        Shape.circle(radius = 10),
        fill_color = linrgb(1, 0, 0))

    instructions = Instructions();

    instruction_text = TextStimulus(
        "Prett button when you see a RARE target",
        font_size = 100,
        fill_color = linrgb(0, 0, 0, 1))


    ready_text = TextStimulus(
        "Get ready!",
        font_size = 150,
        fill_color = linrgb(1, 0, 0, 1))

    correct_text = TextStimulus(
        "Correct!",
        font_size = 150,
        fill_color = linrgb(0, 1, 0, 1))

    incorrect_text = TextStimulus(
        "Incorrect!",
        font_size = 150,
        fill_color = linrgb(1, 0, 0, 1))


    stim1 = PatternStimulus(
        Shape.circle(radius = "3deg"),
        x = "-10deg", y = "-5deg",
        pattern = "stripes",
        pattern_rotation = 90,
        stroke_color = linrgb(0, 0, 0),
        stroke_width = 5,
        fill_color = linrgb(0.50, 0.50, 0.50))

    stim2 = PatternStimulus(
        Shape.circle(radius = "3deg"),
        x = "-8deg", y = "5deg",
        pattern = "stripes",
        stroke_color = linrgb(0, 0, 0),
        stroke_width = 5,
        fill_color = linrgb(0.50, 0.50, 0.50))

    stim3 = PatternStimulus(
        Shape.circle(radius = "3deg"),
        x = "10deg", y = "-5deg",
        pattern = "stripes",
        stroke_color = linrgb(0, 0, 0),
        stroke_width = 5,
        fill_color = linrgb(0.50, 0.50, 0.50))

    stim4 = PatternStimulus(
        Shape.circle(radius = "3deg"),
        x = "8deg", y = "5deg",
        pattern = "stripes",
        stroke_color = linrgb(0, 0, 0),
        stroke_width = 5,
        fill_color = linrgb(0.50, 0.50, 0.50))

    stims = [stim1, stim2, stim3, stim4]

    for i in range(240):
        frame = main_window.get_frame()
        instructions.draw(frame)
        main_window.present(frame)

    for i in range(160):
        frame = main_window.get_frame()
        frame.draw(ready_text)
        main_window.present(frame)

    ready_text.animate("font_size", 500, 1.0)
    ready_text.animate("alpha", 0.0, 0.2)

    for i in range(200):
        frame = main_window.get_frame()
        frame.draw(ready_text)
        frame.draw(fixation)
        main_window.present(frame)

    # listen for key presses
    key_receiver = main_window.create_event_receiver()

    while True:
        i_stim = stims[np.random.randint(0, 4)]
        is_rare = np.random.rand() < 0.1

        if is_rare:
            i_stim["pattern_rotation"] = 90
        else:
            i_stim["pattern_rotation"] = 0

        for i in range(fixation_duration_frames):
            frame = main_window.get_frame()
            frame.draw(fixation)

            main_window.present(frame)

        # clear the key receiver
        key_receiver.poll()

        for i in range(trial_duration_frames):
            frame = main_window.get_frame()

            frame.draw(i_stim)
            frame.draw(fixation)

            main_window.present(frame)

        for i in range(fixation_duration_frames):
            frame = main_window.get_frame()
            frame.draw(fixation)
            main_window.present(frame)

            keys = key_receiver.poll()


            if "Space" in keys.keys_pressed():
                if is_rare:
                    correct_text.animate("alpha", 0.0, 0.2)
                    for i in range(fixation_duration_frames):
                        frame = main_window.get_frame()
                        frame.draw(correct_text)
                        main_window.present(frame)
                else:
                    for i in range(fixation_duration_frames):
                        frame = main_window.get_frame()
                        frame.draw(incorrect_text)
                        main_window.present(frame)
                break


if __name__ == "__main__":
    run_experiment(my_experiment)
