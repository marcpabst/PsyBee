# Copyright (c) 2024 Marc Pabst
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from psychophysics_py import *
import numpy as np

# Create an experiment manager
em = ExperimentManager()

# Get a monitor (0 is usually the internal screen, 1 the first external monitor, etc.)
monitor = em.get_available_monitors()[-1]

# Define the experiment
def my_experiment(wm):

    # create a window
    window = wm.create_default_window()

    # receive keyboard input from the window
    kb = window.create_physical_input_receiver()

    # create a stimulus
    #stim = VideoStimulus(window, Rectangle.fullscreen(), "/Users/marc/Downloads/Movies/BigBuckBunny.mp4", 3840, 2160)


    # create a Gabor patch stimulus
    stim = GaborStimulus(window, Rectangle.fullscreen(), 0, Pixels(20), (0.,0.,0.))


    # loop until the window is closed
    i = 0
    while True:
        i += 1

        # check for keyboard input
        keys = kb.get_inputs()

       
        if i % 10 == 0:
            stim.set_phase(stim.get_phase() + np.pi)
            stim.set_cycle_length(Pixels(i))

        if keys.key_pressed("s"):
            stim.pause()
        
        if keys.key_pressed("p"):
            stim.play()

        frame = window.get_frame()
        frame.add(stim)
        window.submit_frame(frame)

# Run the experiment 
em.run_experiment(my_experiment)