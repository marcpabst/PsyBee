from psydk import run_experiment
from psydk.visual.geometry import Transformation2D, Shape
from psydk.visual.stimuli import ShapeStimulus, GaborStimulus, ImageStimulus, PatternStimulus
import pymunk
import sys

class Ball:
    def __init__(self, x, y, radius, mass, space, randomize=True):
        inertia = pymunk.moment_for_circle(mass, 0, radius, (0, 0))
        self.body = pymunk.Body(mass, inertia)
        self.body.position = x, y
        # if randomize, add a random velocity
        if randomize:
            self.body.apply_impulse_at_local_point((0.1, 0))
        self.shape = pymunk.Circle(self.body, radius, (0, 0))
        self.shape.elasticity = 0.3
        self.shape.friction = 0.8
        space.add(self.body, self.shape)
        self.stim = ShapeStimulus(Shape.circle(radius), x=x, y=y, fill_color=(1, 0, 0, 1))
        self.space = space

    def update(self):
        self.stim["x"] = self.body.position.x
        self.stim["y"] = self.body.position.y

    def remove(self):
        self.space.remove(self.body, self.shape)

def my_experiment(exp_manager) -> None:
    # create a new window
    #
    print(__renderer_factory)

    main_window = exp_manager.create_default_window(fullscreen=True, monitor=1)

    space = pymunk.Space()      # Create a Space which contain the simulation
    space.gravity = 0,98.1 * 2      # Set its gravity
    space.damping = 0.999

    balls = []

    main_window.add_event_handler("mouse_button_press", lambda e: balls.append(Ball(e.position[0], e.position[1], 25, 1, space)))

    # add the ground
    ground_x, ground_y, ground_width = -250, 500, 500
    shape = pymunk.Segment(space.static_body, (ground_x, ground_y), (ground_x + ground_width, ground_y), 0.0)
    shape.elasticity = 0.9999999
    shape.friction = 0.9
    space.add(shape)

    ground_stim = ShapeStimulus(Shape.line(ground_x, ground_y, ground_x + ground_width, ground_y), stroke_color=(0, 0, 0, 1))

    # the background
    # background = PatternStimulus(Shape.rectangle(width="1sw", height="1sh"), x="-0.25sw", y=-550, pattern="checkerboard", stroke_color="red", fill_color=(0.0, 0.0, 0.0, 1.0))

    background = PatternStimulus(Shape.circle(600), pattern="checkerboard", stroke_color="red", fill_color=(0.0, 0.0, 0.0, 1.0))


    for i in range(10000000):
        frame = main_window.get_frame()

        frame.draw(background)

        for i, ball in enumerate(list(balls)):
            # check if the ball is out of the screen
            if ball.body.position.y < -500:
                space.remove(ball.body, ball.shape)
                balls.pop(i)
                continue
            ball.update()
            frame.draw(ball.stim)

        space.step(0.02)


        frame.draw(ground_stim)


        main_window.present(frame)

if __name__ == "__main__":
    run_experiment(my_experiment)
