from psybee import run_experiment
from psybee.visual.geometry import Transformation2D, Shape
from psybee.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus
import pymunk
import sys

def my_experiment(exp_manager) -> None:
    # create a new window

    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    space = pymunk.Space()      # Create a Space which contain the simulation
    space.gravity = 0,98.1      # Set its gravity
    space.damping = 0.999

    mass = 10
    radius = 25
    inertia = pymunk.moment_for_circle(mass, 0, radius, (0, 0))
    body = pymunk.Body(mass, inertia)
    body.position = (0, -500)
    shape = pymunk.Circle(body, radius, (0, 0))
    shape.elasticity = 0.65
    shape.friction = 0.9
    space.add(body, shape)

    # add the ground
    ground_x, ground_y, ground_width = -250, 500, 500
    shape = pymunk.Segment(space.static_body, (ground_x, ground_y), (ground_x + ground_width, ground_y), 0.0)
    shape.elasticity = 0.9999999
    shape.friction = 0.9
    space.add(shape)

    poly_stim = ShapeStimulus(Shape.circle(0, 0, 25), x=-100, fill_color=(1, 0, 0, 1))
    ground_stim = ShapeStimulus(Shape.line(ground_x, ground_y, ground_x + ground_width, ground_y), stroke_color=(0, 0, 0, 1))


    for i in range(10000000):
        frame = main_window.get_frame()

        space.step(0.02)
        poly_stim["x"], poly_stim["y"] = body.position

        frame.draw(poly_stim)
        frame.draw(ground_stim)

        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
