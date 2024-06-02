"""A simple example experiment using the psybee library."""

import logging
import os
import time

from .bubble_simulation import BubbleSimulation
from psybee import MainLoop, ExperimentManager
from psybee.audio import AudioDevice, FileStimulus
from psybee.geometry import Circle, Rectangle, Pixels, ScreenWidth
from psybee.events import EventKind
from psybee.stimuli import GaborStimulus, ImageStimulus, SpriteStimulus
from psybee.window import Window

n_balls = 4
n_init_steps = 100000


def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    window = exp_manager.create_default_window()

    (width, height) = (window.width_px, window.height_px)

    print(f"Window size: {width}x{height}")  # noqa: T201

    sim = BubbleSimulation(width=width, height=height, n_balls=n_balls, n_init_steps=n_init_steps)

    resources = os.path.join(os.path.dirname(__file__), "resources")  # noqa: PTH118, PTH120

    kb = window.create_event_receiver()

    default_audio_device = AudioDevice()

    audio_miss = FileStimulus(default_audio_device, os.path.join(resources, "bubbles.mp3"))  # noqa: PTH118
    audio_hit = FileStimulus(default_audio_device, os.path.join(resources, "collect.mp3"))  # noqa: PTH118

    ball_stims = []

    for _ in range(n_balls):
        stim = GaborStimulus(window, Circle(Pixels(0), Pixels(0), ScreenWidth(0.05)), 0, Pixels(20), ScreenWidth(0.01),
                             ScreenWidth(0.01), 0, (0.0, 0.0, 0.0))

        ball_stims.append(stim)

    crosshair = ImageStimulus(
        window,
        Rectangle(Pixels(-50), Pixels(-50), Pixels(100), Pixels(100)),
        os.path.join(resources, "crosshair.png"),  # noqa: PTH118
    )
    # hide by default
    crosshair.hide()

    bubbles = SpriteStimulus(
        window,
        Rectangle(ScreenWidth(-0.05), ScreenWidth(-0.05), ScreenWidth(0.1), ScreenWidth(0.1)),
        os.path.join(resources, "buble_pop_two_spritesheet_512px_by512px_per_frame.png"),
        4,
        2,
        fps=20,
        repeat=1,
    )

    sparkle = SpriteStimulus(
        window,
        Rectangle(ScreenWidth(-0.05), ScreenWidth(-0.05), ScreenWidth(0.1), ScreenWidth(0.1)),
        os.path.join(resources, "sparkle_spritesheet_256px_by_256px_per_frame.png"),
        3,
        3,
        fps=20,
        repeat=1,
    )

    # subject_id = exp_manager.prompt("Press any key to start the experiment")

    # print(f"Subject ID: {subject_id}")  # noqa: T201

    window.stimuli.extend(ball_stims)
    window.stimuli.append(crosshair)
    window.stimuli.append(bubbles)
    window.stimuli.append(sparkle)

    # hide the cursor
    window.cursor_visible = False

    def click_handler(event):
        hit = False
        for stim in ball_stims:
            if stim.visible and stim.contains(*event.position):
                stim.hide()
                hit = True

        if hit:
            audio_hit.restart()
            sparkle.set_translation(*event.position)
            sparkle.reset()

        else:
            audio_miss.restart()
            bubbles.set_translation(*event.position)
            bubbles.reset()

    def mouse_move_handler(event):
        crosshair.show()
        crosshair.set_translation(*event.position)

    # add event handlers
    window.add_event_handler(EventKind.CURSOR_MOVED, mouse_move_handler)
    window.add_event_handler(EventKind.MOUSE_BUTTON_PRESS, click_handler)
    window.add_event_handler(EventKind.TOUCH_START, click_handler)

    while True:
        frame = window.get_frame()
        frame.set_bg_color((0.5, 0.5, 0.5))

        # advance the simulation
        new_pos = next(sim)

        for i, stim in enumerate(ball_stims):
            stim.set_translation(Pixels(new_pos[i][0]), Pixels(new_pos[i][1]))

        window.present(frame)


if __name__ == "__main__":
    # Set the logging level
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"

    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.INFO)

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
