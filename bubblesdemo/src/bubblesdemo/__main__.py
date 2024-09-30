#!/usr/bin/env python

import time
import pathlib
from psybee import ExperimentManager, WindowOptions, MainLoop, GaborStimulus, ImageStimulus, SpriteStimulus, Transformation2D, Image, GaborStimulus, Size, TextStimulus, Rgba
from bubble_simulation import BubbleSimulation
import numpy as np
import random

# setup the experiment parameters
exp_params = {
    "n_bubbles" : 1, # number of bubbles visible on the screen at the same time
    "duration_bubbles" : 8.0, # max. duration in seconds a bubble is visible on the screen
    "duration_bubbles_jitter" : 0.0, # uniform jitter in seconds for the duration of the bubbles
    "duration_delay" : 3.0, # duration in seconds until the next bubble
    "duration_delay_jitter" :2.0, # uniform jitter in seconds for the delay until the next bubble
    "n_init_steps" : 10000, # number of initial simulation steps
}


sprites = {
    "fishes": [
        Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/fish/fish_puffer.png", 4, 3),
        Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/fish/fish_green.png", 8, 1),
        Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/fish/fish_purple.png", 8, 1),
        Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/fish/fish_puffer_deflated.png", 4, 3),
        ],
    "bubbles": Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/misc/bubbles.png", 4, 2),
    "octopus": Image.from_spritesheet("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/fish/shark2.png", 4,3),
}

def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202
    print("Running the experiment")

    # create a window
    window = exp_manager.create_default_window()
    print("Window created (py)")

    bg = ImageStimulus("/Users/marc/psybee/bubblesdemo/src/bubblesdemo/resources/bg/underwater2.png", 0, 0, "1sw", "1sh")

    start_text = TextStimulus(0, "-0.3sh", "SharkAttack", "3cm", fill = Rgba(0.055, 0, 0.871, 1.0))
    start_text2 = TextStimulus(0, "0.2sh", "Tap to start!", "2cm", fill = Rgba(1, 1, 1, 1.0))

    octopus = SpriteStimulus(sprites["octopus"], 15, 0, 0, "0.4sw", "0.3sw")

    fish_stims = []

    game_active = False


    # wait for 1s
    #time.sleep(1)

    (w, h) = window.get_size()


    game = BubbleSimulation(200, area_width=w, area_height=h, n_bubbles=0, n_init_steps=exp_params["n_init_steps"],
                            duration_bubbles=exp_params["duration_bubbles"],
                            duration_bubbles_jitter=exp_params["duration_bubbles_jitter"],
                            duration_delay=exp_params["duration_delay"],
                            duration_delay_jitter=exp_params["duration_delay_jitter"])

    game.add_bubble()
    game.add_bubble()
    game.add_bubble()

    def start_game(event):
        if not game.is_running:
            game.run()

    def mouse_click_handler(event):
        print("Mouse click at", event.position)

        if target_pos := game.check_position(event.position, event.window):

            now = time.time()
            # add bubbles
            bubbles_stim = SpriteStimulus(sprites["bubbles"], 30, target_pos[0], target_pos[1], 300, 300, repeat=False)
            fish_stims.append((bubbles_stim, now))

            # select a random fish
            fish_imgs = random.choice(sprites["fishes"])

            # move fish to the left or right, depending on which is closer to the edge of the screen
            if target_pos[0].eval(event.window) > 0:
                fish = SpriteStimulus(fish_imgs, 15, target_pos[0], target_pos[1], -200, 200)
                fish.animate("x", target_pos[0] + Size("1sw"), 2.0)
            else:
                fish = SpriteStimulus(fish_imgs, 15, target_pos[0], target_pos[1], 200, 200)
                fish.animate("x", target_pos[0] - Size("1sw"), 2.0)


            fish_stims.append((fish, now))





    # add event handlers
    window.add_event_handler("MouseButtonPress", start_game)
    window.add_event_handler("MouseButtonPress", mouse_click_handler)

    bg.animate("opacity", 0.5, 2.0)
    # start loop
    while not game.is_running:
        frame = window.get_frame() # get a frame

        frame.draw(bg)
        frame.draw(start_text)
        frame.draw(start_text2)
        frame.draw(octopus)

        window.present(frame)

    while True:
        frame = window.get_frame() # get a frame
        # remove fish that are older than 1s
        fish_stims = [(f, t) for (f, t) in fish_stims if time.time() - t < 2.0]

        for fish, _ in fish_stims:
            frame.draw(fish)

        for b in next(game):
            frame.draw(b)

        window.present(frame)



if __name__ == "__main__":

    # Run the experiment
    MainLoop().run_experiment(my_experiment)
