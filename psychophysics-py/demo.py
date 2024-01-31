# Copyright (c) 2024 Marc Pabst
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import psychophysics_py as psy

# Create ExperimentManager
em = psy.ExperimentManager()
mons = em.get_available_monitors()
print(mons)

win_opts = psy.WindowOptions()


def my_experiment(window):
    shape1 = psy.ShapeStimulus(window, (1, 0, 0))
    shape2 = psy.ShapeStimulus(window, (0, 1, 0))
    current_shape = shape1
    print(window)

    for i in range(10000):
        frame = window.get_frame()
        frame.add(current_shape)
        window.submit_frame(frame)

        # every 10 frames, switch the shape
        if i % 10 == 0:
            if current_shape == shape1:
                current_shape = shape2
            else:
                current_shape = shape1


# Run the experiment 
em.run_experiment(win_opts, my_experiment)

    