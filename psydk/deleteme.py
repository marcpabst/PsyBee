from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus
import sys

def my_experiment(exp_manager) -> None:
    # create a new window

    main_window = exp_manager.create_default_window(fullscreen=True, monitor=2)

    input_circles = {}


    def circle_move(event):
        print(event)
        x, y = event.position
        id = event.id if event.id is not None else 0

        if id not in input_circles:
            input_circles[id] = ShapeStimulus(Shape.circle(0, 0, "0.01sw"), x=-100, fill_color=(1, 0, 0, 1))

        input_circles[id]["x"] = x
        input_circles[id]["y"] = y

    # def circle_click(event):
    #     circle["fill_color"] = "yellow"

    # def circle_release(event):
    #     circle["fill_color"] = "blue"

    main_window.add_event_handler("key_press", lambda e: sys.exit(0) if e.key == "Q" else None)
    main_window.add_event_handler("cursor_moved", circle_move)
    main_window.add_event_handler("touch_move", circle_move)
    # main_window.add_event_hanxdler("mouse_button_press", circle_click)
    # main_window.add_event_handler("mouse_button_release", circle_release)

    event_receiver = main_window.create_event_receiver()

    # rect0 = ShapeStimulus(Shape.rectangle("-0.5sw", "-0.5sh", "1sw", "0.5sh"), fill_color=(1, 0, 0, 1))
    rect1 = ShapeStimulus(Shape.rectangle("-0.5sw", "-0.5sh", "1sw", "1sh"), fill_color=(0, 0, 0, 1))
    image = ImageStimulus("test.png", "-0.25sw", "-0.25sh", "0.25sw", "0.25sw", anchor = "center")
    rect2 = ShapeStimulus(Shape.rectangle(0, 0, "0.25sw", "0.25sw"), stroke_color=(1, 0, 0, 1), stroke_width=10)
    gabor = GaborStimulus(0, 0, "0.25sw", 70, 50, anchor = "center", stroke_style="Solid", stroke_width=5)

    is_visible = False
    is_visible2 = False

    for i in range(10000000):
        frame = main_window.get_frame()


        angle = (i / 10) % 360

        gabor.rotated_at(angle, 0, 0)

        rect2.rotated_at(-angle, 0, 0)
        # image.rotated_at(-angle, 0, 0)


        keys = event_receiver.poll()

        if "Enter" in keys.keys_pressed():
            is_visible = True

        if "Enter" in keys.keys_released():
            is_visible = False

        if is_visible:
            frame.draw(rect1)

        # if is_visible2:
        #     frame.draw(rect0)

        frame.draw(gabor)
        frame.draw(image)
        frame.draw(rect2)

        for circle in input_circles.values():
            frame.draw(circle)

        is_visible2 = not is_visible2

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
