#!/usr/bin/env python

import time
import pathlib
import numpy as np
from psybee import ExperimentManager, WindowOptions, MainLoop, GaborStimulus, Size, Transformation2D, ImageStimulus, SpriteStimulus, RawRgba
from bubble_simulation import BubbleSimulation


# setup the experiment parameters
exp_params = {
    "n_bubbles" : 1, # number of bubbles visible on the screen at the same time
    "duration_bubbles" : 8.0, # max. duration in seconds a bubble is visible on the screen
    "duration_bubbles_jitter" : 0.0, # uniform jitter in seconds for the duration of the bubbles
    "duration_delay" : 3.0, # duration in seconds until the next bubble
    "duration_delay_jitter" :2.0, # uniform jitter in seconds for the delay until the next bubble
    "n_init_steps" : 10000, # number of initial simulation steps
}

# find path to ./res folder
res_path = pathlib.Path(__file__).parent / "res"






def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202

    # create a window
    window = exp_manager.create_default_window()

    # create a bubble simulation
    game = BubbleSimulation(200, area_width=3024, area_height=1890, n_bubbles=0, n_init_steps=exp_params["n_init_steps"])

    # setup game components
    crosshair = ImageStimulus(Size.Pixels(50), Size.Pixels(50), str(res_path / "crosshair.png"))
    touch_animation = SpriteStimulus(Size.Pixels(300), Size.Pixels(300), str(res_path / "buble_pop_two_spritesheet_512px_by512px_per_frame.png"), 4, 2, 25, 1)
    touch_animation.hide()

    # bubbles will store the bubbles on the screen and their start time
    bubbles = []

    def mouse_move_handler(event):

        pos = event.position
        #crosshair.set_transformation(Transformation2D.Translation(-pos[0]-Size.Pixels(25), -pos[1]-Size.Pixels(25)))

    def mouse_click_handler(event):
        pos = event.position

        for (i, bubble) in enumerate(bubbles):

            # check if the bubble was touched/clicked
            if bubble["stim"].contains(pos[0], pos[1], window) and bubble["stim"].visible:

                # if yes, remove the ball
                bubble["stim"].hide()
                sim.remove_bubble(bubble["handle"])
                bubble["t_end"] = time.time()

                # play the touch animation
                touch_animation.set_transformation(Transformation2D.Translation(-pos[0]-Size.Pixels(150), -pos[1]-Size.Pixels(150)))
                touch_animation.reset()
                touch_animation.show()

    # add event handlers
    window.add_event_handler("CursorMoved", mouse_move_handler)
    window.add_event_handler("MouseButtonPress", mouse_click_handler)

    t_start = time.time()

    # create all the bubbles
    for i in range(exp_params["n_bubbles"]):  add_bubble(sim, bubbles)

    # main loop
    for i in range(int(1e6)):

        frame = window.get_frame() # get a frame
        frame.bg_color = RawRgba(.5, .5, .5, 1) # set the background color

        # get the position of the ball
        new_positions = next(sim)

        for j, bubble in enumerate(list(bubbles)):

            # check if the bubble has timed out
            if bubble["stim"].visible and time.time() - bubble["t_start"] > bubble["duration"]:
                bubble["stim"].hide()
                sim.remove_bubble(bubble["handle"])
                bubble["t_end"] = time.time()

            # check if the bubble should be removed
            if "t_end" in bubble:
                if time.time() - bubble["t_end"] > bubble["delay"]:

                    # remove the bubble from the list
                    del bubbles[j]

                    add_bubble(sim, bubbles)


            # for all visible bubbles, update their position
            if bubble["stim"].visible:

                # move
                stim = bubble["stim"]
                handle = bubble["handle"]
                pos = new_positions[handle]

                stim.set_transformation(Transformation2D.Translation(Size.Pixels(pos[0]), Size.Pixels(pos[1])))
                frame.add(stim)

        #frame.add(crosshair)
        frame.add(touch_animation)

        window.present(frame)


if __name__ == "__main__":

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
