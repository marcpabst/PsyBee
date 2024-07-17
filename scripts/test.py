#!/usr/bin/env python3

from psybee import ExperimentManager, WindowOptions, MainLoop, GaborStimulus, Size, Transformation2D, ImageStimulus, SpriteStimulus
from bubble_simulation import BubbleSimulation

n_balls = 2
n_init_steps = 0

def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    window = exp_manager.create_default_window()

    sim = BubbleSimulation(width=1920, height=1080, n_balls=n_balls, n_init_steps=n_init_steps)

    crosshair = ImageStimulus( Size.Pixels(50), Size.Pixels(50), "crosshair.png")
    bubbles = SpriteStimulus(Size.Pixels(100), Size.Pixels(100), "buble_pop_two_spritesheet_512px_by512px_per_frame.png", 4, 2, 25, 1)

    ball_stims = []

    def mouse_move_handler(event):
        pos = event.position
        crosshair.set_transformation(Transformation2D.Translation(-pos[0]-Size.Pixels(25), -pos[1]-Size.Pixels(25)))

    def mouse_click_handler(event):
        pos = event.position
        bubbles.set_transformation(Transformation2D.Translation(-pos[0]-Size.Pixels(50), -pos[1]-Size.Pixels(50)))
        bubbles.reset()

    # add event handlers
    window.add_event_handler("CursorMoved", mouse_move_handler)
    window.add_event_handler("MouseButtonPress", mouse_click_handler)

    # create a GaborStimulus
    for _ in range(n_balls):
        stim = GaborStimulus(Size.Pixels(100), 100.0,0.1,0,0)

        ball_stims.append(stim)


    for i in range(int(1e6)):
        frame = window.get_frame()


        # get the position of the ball
        new_pos = next(sim)

        for j, stim in enumerate(ball_stims):
            frame.add(stim)

            print(new_pos[j])

            # move the ball
            stim.set_transformation(Transformation2D.Translation(Size.Pixels(new_pos[j][0]), Size.Pixels(new_pos[j][1])))



        frame.add(crosshair)
        frame.add(bubbles)

        window.present(frame)


if __name__ == "__main__":

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
