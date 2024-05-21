# Basic Concepts

## Experiment

We use the term "experiment" to refer to a self-contained program that presents some sort stimuli to a participant and records their responses. In Psychophysics, an experiment is represented by an "experiment function" that takes an `ExperimentManager` as an argument. The `ExperimentManager` provides access to the display, input devices, and other resources that are necessary for running an experiment.

## Window

In Psychophysics, a "window" is a visual display that is used to present stimuli to a participant. Windows can be created in different sizes and resolutions, and can be used to present stimuli in different color spaces. A window is always preseted on a (physical) monitor, but does not necessarily cover the entire monitor (however, for most experiments and technical reasons, you will want to use the entire monitor).

## Stimuli

A "stimulus" is a visual or auditory object that is presented to a participant. Stimuli can be simple, like a colored square, or complex, like a video or a sound.

## Frame

A "frame" is a single image that is presented to a participant. The pace at which frames are presented to the participant is called the "frame rate". The frame rate is usually measured in "frames per second" (FPS). The frame rate is important because it determines the temporal resolution of the experiment. For example, if you want to present a stimulus for 100 ms, you need to present 10 frames at a frame rate of 100 FPS. Most monitors support a number of different frame rates, but the most common frame rate is 60 FPS.

Note that most monitors have a fixes frame rate, which means that you can only present stimuli with a duration that is a multiple of the frame duration. For example, if the frame rate is 60 FPS, you can only present stimuli for 16.67 ms, 33.33 ms, 50 ms, 66.67 ms, and so on. 

> Some monitors support "variable refresh rates" (VRR), which means that the frame rate can be changed dynamically. Nvidia calls this "G-Sync", AMD calls this "FreeSync", and the VESA standard is called "Adaptive-Sync". VRR is useful because it allows you to present stimuli with arbitrary durations, which is important for some types of experiments. However, VRR is currently not supported by `psychophysics-rs` as it comes with a number of technical challenges and sometimes poor driver support.

In `psychophysics-rs`, a frame is represented by the `Frame` struct. You obtain an empty frame from a `Window` and then add stimuli to it. The frame is then presented to the participant by calling the `present` method on the window.

The time it takes from submitting a frame to the window to the frame being displayed on the monitor is called the "latency". The latency is usually measured in "milliseconds" (ms). 
