from psychophysics_py import *


# Define the experiment
def my_experiment(wm):
    # create a window
    window = wm.create_default_window()

    # receive keyboard input from the window
    kb = window.create_physical_input_receiver()

    stim1 = GaborStimulus(
        window,
        Circle(Pixels(0), Pixels(0), Pixels(100)),
        0,
        Pixels(30),
        Pixels(40),
        Pixels(40),
        0,
        (0.0, 0.0, 0.0),
    )

    stim2 = ImageStimulus(
        window,
        Rectangle(Pixels(-230), Pixels(-150), Pixels(460), Pixels(300)),
        "/Users/marc/psychophysics/rustacean-flat-noshadow.png",
    )

    stim3 = SpriteStimulus(
        window,
        Rectangle(
            ScreenWidth(-0.1), ScreenWidth(-0.1), ScreenWidth(0.2), ScreenWidth(0.2)
        ),
        "/Users/marc/psychophysics/white-sails-rocking-action-25-frames-1317px-by-1437px-per-frame.png",
        5,
        5,
    )

    while True:
        for i in range(100):
            stim1.set_orientation(stim1.orientation() + 0.01)
            frame = window.get_frame()
            frame.add(stim1)
            window.submit_frame(frame)

        for i in range(100):
            frame = window.get_frame()
            frame.add(stim2)
            window.submit_frame(frame)

        for i in range(100):
            stim3.advance_image_index()
            frame = window.get_frame()
            frame.add(stim3)
            window.submit_frame(frame)


if __name__ == "__main__":

    # Create an experiment manager
    em = ExperimentManager()

    # Get a monitor (0 is usually the internal screen, 1 the first external monitor, etc.)
    monitor = em.get_available_monitors()[-1]

    # Run the experiment
    em.run_experiment(my_experiment)
