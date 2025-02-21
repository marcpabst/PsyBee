from psybee import run_experiment, ShapeStimulus, Shape, GaborStimulus, ImageStimulus, Transformation2D
import time
import sys

def my_experiment(exp_manager) -> None:
    # create a new window

    main_window = exp_manager.create_default_window(0)

    main_window.add_event_handler("KeyPress", lambda e: sys.exit(0) if e.key == "Q" else None)

    event_receiver = main_window.create_event_receiver()


    rect = ShapeStimulus(Shape.rectangle(-400, -400, 800, 800), fill_color=(0, 0, 0, 1))
    image = ImageStimulus("test.png", -300, -300, main_window, 400, 400, anchor = "center")
    rect2 = ShapeStimulus(Shape.rectangle(-300, -300, 400, 400), stroke_color=(1, 0, 0, 1), stroke_width=10)
    gabor = GaborStimulus(0, 0, 500, 70, 50, anchor = "center")

    is_visible = False

    for i in range(10000000):
        frame = main_window.get_frame()

        angle = (i / 10) % 360

        gabor.rotated_at(angle, 0, 0)

        rect2.rotated_at(-angle, 0, 0)
        image.rotated_at(-angle, 0, 0)

        frame.draw(gabor)
        frame.draw(image)
        frame.draw(rect2)

        keys = event_receiver.poll()

        if "Enter" in keys.keys_pressed():
            is_visible = True

        if "Enter" in keys.keys_released():
            is_visible = False



        if is_visible:
            frame.draw(rect)

        main_window.present(frame)






if __name__ == "__main__":
    run_experiment(my_experiment)
