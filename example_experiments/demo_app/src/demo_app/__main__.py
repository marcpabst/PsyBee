"""A simple example experiment using the psychophysics_py library."""

import logging
import os

from bubble_simulation import BubbleSimulation
from psychophysics_py import (
    AudioDevice,
    Circle,
    EventData,
    ExperimentManager,
    ExperimentManagerOld,
    FileStimulus,
    GaborStimulus,
    ImageStimulus,
    Pixels,
    Rectangle,
    ScreenWidth,
    SpriteStimulus,
)

n_balls = 4
n_init_steps = 10000


def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # set-up the simulation
    sim = BubbleSimulation()

    # create a window
    window = exp_manager.create_default_window()

    resources = os.path.join(os.path.dirname(__file__), "resources")  # noqa: PTH118, PTH120

    kb = window.create_event_receiver()

    default_audio_device = AudioDevice()

    audio_stim1 = FileStimulus(default_audio_device, "./resources/bubles.mp3")

    ball_stims = []

    for _ in range(n_balls):
        stim = GaborStimulus(
            window,
            Circle(Pixels(0), Pixels(0), ScreenWidth(0.05)),
            0,
            Pixels(20),
            ScreenWidth(0.01),
            ScreenWidth(0.01),
            0,
            (0.0, 0.0, 0.0),
        )

        ball_stims.append(stim)

    stim2 = ImageStimulus(
        window,
        Rectangle(Pixels(-50), Pixels(-50), Pixels(100), Pixels(100)),
        os.path.join(resources, "crosshair.png"),  # noqa: PTH118
    )

    stim3 = SpriteStimulus(
        window,
        Rectangle(ScreenWidth(-0.05), ScreenWidth(-0.05), ScreenWidth(0.1), ScreenWidth(0.1)),
        "resources/buble_pop_two_spritesheet_512px_by512px_per_frame.png",
        4,
        2,
        fps=20,
        repeat=1,
    )

    subject_id = exp_manager.prompt("Press any key to start the experiment")

    print(f"Subject ID: {subject_id}")  # noqa: T201

    window.stimuli.extend(ball_stims)
    window.stimuli.append(stim2)
    window.stimuli.append(stim3)

    stim3.reset()

    last_mouse_pos = (Pixels(0), Pixels(0))

    while True:
        frame = window.get_frame()
        frame.set_bg_color((0.5, 0.5, 0.5))

        # advance the simulation
        new_pos = next(sim)

        for i, stim in enumerate(ball_stims):
            # move the stimulufs
            stim.set_translation(Pixels(new_pos[i][0]), Pixels(new_pos[i][1]))

        window.submit_frame(frame)

        # check for new events
        events = kb.poll()
        for i in range(len(events)):
            event = events[i]
            data = event.data

            if isinstance(data, EventData.CursorMoved):
                # update the position of the image stimulus
                stim2.set_translation(data.position[0], data.position[1])

                # update the position of the mouse
                last_mouse_pos = data.position

            if isinstance(data, EventData.MouseButtonPress):
                # play the audio stimulus
                audio_stim1.reset()
                audio_stim1.play()

                # move stimulus 3 to the position of the mouse
                stim3.set_translation(last_mouse_pos[0], last_mouse_pos[1])

                # reset the sprite
                stim3.reset()


if __name__ == "__main__":
    # Set the logging level
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"

    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.INFO)

    # Create an experiment manager
    em = ExperimentManagerOld()

    # Get a monitor (0 is usually the internal screen, 1 the first external monitor, etc.)
    monitor = em.get_available_monitors()[-1]

    # Run the experiment
    em.run_experiment(my_experiment)
