# Copyright (c) 2024 Marc Pabst
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import threading
import psychophysics_py as psy

# Create an experiment manager
em = psy.ExperimentManager()
# Get a monitor (0 is usually the internal screen, 1 the first external
# monitor, etc.)
monitor = em.get_available_monitors()[-1]

some_global_variable = 0

# Use the full screen of the second monitor at 60 Hz, select the highest
# resolution available
win_opts = psy.WindowOptions(mode = "fullscreen_highest_resolution", 
                             monitor = monitor, 
                             refresh_rate = 60)

# Define the experiment
def my_experiment(window):
     # print the thread id
    colors = [(1, 0, 0), (0, 0, 1)]
    color_index = 0
    stim = psy.ShapeStimulus(window, (0, 0, 0))

    for i in range(100000):
        frame = window.get_frame()
        frame.add(stim)
        window.submit_frame(frame)

       

        color_index = (color_index + 1) % len(colors)
        stim.set_color(colors[color_index])

# Run the experiment 
em.run_experiment(win_opts, my_experiment)