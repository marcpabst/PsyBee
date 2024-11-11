#!/usr/bin/env python

import time
import pathlib
from psybee import ExperimentManager, WindowOptions, MainLoop, GaborStimulus, ImageStimulus, SpriteStimulus, Transformation2D, Image, GaborStimulus, Size, TextStimulus, Rgba
import numpy as np

import pandas as pd
import random
from .arkit_eyetracking import ARKitEyeTracker
from .image_filters import log_gabor_filter

try:
    from rubicon.objc import ObjCClass

    NSFileManager = ObjCClass("NSFileManager")

    # find the path to document using URL.documentsDirectory
    documents_path = NSFileManager.defaultManager.URLsForDirectory_inDomains_(9, 1)[0].path
except:
    print("cant load objc")
    documents_path = "."

# find the path to the resources
resources_path = str(pathlib.Path(__file__).parent / "resources")

sprites = {
    "fishes": [
        Image.from_spritesheet(resources_path + "/fish/fish_puffer.png", 4, 3),
        Image.from_spritesheet(resources_path + "/fish/fish_green.png", 8, 1),
        Image.from_spritesheet(resources_path + "/fish/fish_purple.png", 8, 1),
        Image.from_spritesheet(resources_path + "/fish/fish_puffer_deflated.png", 4, 3),
        ],
    "bubbles": Image.from_spritesheet(resources_path + "/misc/bubbles.png", 4, 2),
    "octopus": Image.from_spritesheet(resources_path + "/fish/octopus.png", 4,4),
}

def my_experiment(exp_manager: ExperimentManager) -> None:
    """Run the experiment."""  # noqa: D202
    print(f"Documents path: {documents_path}")
    # create a window
    window = exp_manager.create_default_window()

    # create an ARKit eye tracker
    eye_tracker = ARKitEyeTracker()
    eye_tracker.run()

    # create a 500 x 500 pixel random noise image
    noise = np.random.rand(2560, 1664)

    # filter the noise
    filtered_noise, filtered_fft, filter = log_gabor_filter(noise, f0=0.001, sigma_f=0.002)

    # add 2 more (so we have 3) color channels
    filtered_noise = np.repeat(filtered_noise[:, :, np.newaxis], 3, axis=2)

    # scale to 0-1
    filtered_noise = (filtered_noise - filtered_noise.min()) / (filtered_noise.max() - filtered_noise.min())

    # make array contiguous
    filtered_noise = np.ascontiguousarray(filtered_noise)

    print(filtered_noise.shape)

    # turn into an Image and move to the gpu
    random_image = Image.from_numpy(filtered_noise)
    random_image.move_to_gpu(window)

    gabor_stim = GaborStimulus(
                        0, 0,  # center x, y
                        "40cm", # size
                        ".5cm",# cycle length
                        "100cm", # sigma
                        orientation = 0,
            )

    text_stim = TextStimulus(0, "0.3sh", "-", "2cm", fill = Rgba(1, 1, 1, 1.0))

    # move all octopus frames to the gpu
    for i in range(len(sprites["octopus"])):
        sprites["octopus"][i].move_to_gpu(window)

    octopus_stim = SpriteStimulus(sprites["octopus"], 15, 0, 0, "0.4sw", "0.35sw")

    random_image_stim = ImageStimulus(random_image, 0, 0, "1sw", "1sw")

    # start a timer
    start_time = time.time()

    # run the experiment for 10 seconds
    while start_time + 10 > time.time():

        # # after 5 seconds, change orientation to np.pi/2
        # if start_time + 5 < time.time():
        #     gabor_stim["orientation"] = np.pi/2

        frame = window.get_frame() # get a frame

        # target degree visual angle
        target_deg = 1.0

        # get the face distance in m
        dist = eye_tracker.get_current_face_distance()

        # calculate object size in m to archieve the target degree visual angle
        object_size_m = 2 * dist * np.tan(np.deg2rad(target_deg / 2))

        # calculate the phase based on current time
        gabor_stim["phase"] = ((time.time() - start_time) * 1.0 * np.pi) % (np.pi)

        # calculate the size of the gabor stimulus
        new_size_cm =object_size_m * 400
        gabor_stim["cycle_length"] = f"{new_size_cm}cm"
        text_stim["text"] = f"{dist*100.0:.1f}cm"

        # draw the gabor stimulus
        # frame.draw(gabor_stim)
        frame.draw(text_stim)



        # draw the random noise image
        frame.draw(random_image_stim)

        # draw the octopus
        frame.draw(octopus_stim)

        window.present(frame)

    # pause the eye tracker
    eye_tracker.pause()

    # wait 1s to avoid race conditions (BAD PRACTICE)
    time.sleep(1)

    df = eye_tracker.as_df()

    # save the dataframe to a csv file
    df.to_csv(f"{documents_path}/face_distances.csv", index=False)

    print(df)

    print("Experiment finished - saved face distances to face_distances.csv")

def tracking_main():
    # Run the experiment
    MainLoop().run_experiment(my_experiment)

if __name__ == "__main__":
    tracking_main()
