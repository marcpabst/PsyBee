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

def my_experiment():
    print("Hello Worldd!")

# Run the experiment 
em.run_experiment(win_opts, my_experiment)

    