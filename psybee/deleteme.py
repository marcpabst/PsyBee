from psybee import run_experiment, ShapeStimulus, Shape, GaborStimulus, ImageStimulus, Transformation2D
import time

def my_experiment(exp_manager) -> None:
    # create a new window

    main_window = exp_manager.create_default_window(0)

    # sleep for 1 second
    time.sleep(0.1)

    rect = Shape.rectangle(500, 500, 400, 400)
    stim = ShapeStimulus(rect, fill_color=(121/255, 165/255, 177/255))
    image = ImageStimulus("test.png", 500, 500, main_window, 400, 400, anchor = "center")

    gabor = GaborStimulus(500, 500, 500, 70, 50, anchor = "center")



    # gabor.translated("-0.5sw", "-0.5sh")

    for i in range(10000000):
        frame = main_window.get_frame()

        angle = (i / 10) % 360

        gabor.rotated_at(angle, 500, 500)
        image.rotated_at(-angle, 500, 500)

        # frame.draw(stim)
        frame.draw(gabor)
        frame.draw(image)

        main_window.present(frame)


    print(rect)

    time.sleep(1000)


if __name__ == "__main__":
    run_experiment(my_experiment)
