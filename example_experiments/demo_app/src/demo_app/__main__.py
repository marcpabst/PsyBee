from psychophysics_py import *
import psychophysics_py as psy
import os
import rapier2d_py
import numpy as np
import time


# logging
import logging


n_balls = 4
n_init_steps = 10000


def correct_velocity(velocity, speed):
    np_velocity = np.array([velocity.x(), velocity.y()])
    np_dir = np_velocity / np.linalg.norm(np_velocity)
    np_velocity = np_dir * speed
    return rapier2d_py.RealVector(np_velocity[0], np_velocity[1])


# create an Iterator that simulates the movement of a stimulus
class BubbleSimulation:
    def __init__(self):
        # set up the simulation
        self.speed = 0.0

        # create sets
        self.collider_set = rapier2d_py.ColliderSet()
        self.rigid_body_set = rapier2d_py.RigidBodySet()

        # create walls for a FHD screen, with (0,0) at the center
        walls_vertices = [
            rapier2d_py.RealPoint(-960.0, -540.0),
            rapier2d_py.RealPoint(960.0, -540.0),
            rapier2d_py.RealPoint(960.0, 540.0),
            rapier2d_py.RealPoint(-960.0, 540.0),
            rapier2d_py.RealPoint(-960.0, -540.0),
        ]

        wall_collider = rapier2d_py.Collider(
            collider_type="polyline",
            vertices=walls_vertices,
            restitution=1.0,  # no energy loss
            friction=0.0,  # no friction
        )

        self.collider_set.insert(wall_collider)

        self.balls = []

        for i in range(n_balls):
            # build a collider
            ball = rapier2d_py.Collider(
                collider_type="ball",
                radius=200.0,
                restitution=1.0,  # no energy loss
                friction=0.0,  # no friction
            )

            # calculate the initial position of the ball (in a grid)
            max_balls_per_row = int(np.sqrt(n_balls))
            initial_position = rapier2d_py.RealVector(
                -800.0 + 1600.0 / (max_balls_per_row + 1) * (i % max_balls_per_row + 1),
                -400.0 + 800.0 / (max_balls_per_row + 1) * (i // max_balls_per_row + 1),
            )

            # build a rigid body
            ball_rigid_body = rapier2d_py.RigidBody(
                body_type="dynamic",
                translation=initial_position,
                linvel=rapier2d_py.RealVector(50.0, 50.0),
            )
            # add rigid body to rigid body set
            rigid_body_handle = self.rigid_body_set.insert(ball_rigid_body)
            self.collider_set.insert_with_parent(
                ball, rigid_body_handle, self.rigid_body_set
            )

            self.balls.append(rigid_body_handle)

        # set-up physics
        self.gravity = rapier2d_py.RealVector(0.0, 0.0)
        self.integration_parameters = rapier2d_py.IntegrationParameters.default()
        self.physics_pipeline = rapier2d_py.PhysicsPipeline()
        self.island_manager = rapier2d_py.IslandManager()
        self.broad_phase = rapier2d_py.DefaultBroadPhase()
        self.narrow_phase = rapier2d_py.NarrowPhase()
        self.impulse_joint_set = rapier2d_py.ImpulseJointSet()
        self.multibody_joint_set = rapier2d_py.MultibodyJointSet()
        self.ccd_solver = rapier2d_py.CCDSolver()
        self.query_pipeline = rapier2d_py.QueryPipeline()

    def __iter__(self):
        return self

    def __next__(self):

        # set velocity
        for i in range(n_balls):
            rigid_body = self.rigid_body_set.get(self.balls[i])
            rigid_body.set_linvel(rapier2d_py.RealVector(0.0, 0.0), True)

        # run physics
        for i in range(100):
            self.physics_pipeline.step(
                self.gravity,
                self.integration_parameters,
                self.island_manager,
                self.broad_phase,
                self.narrow_phase,
                self.rigid_body_set,
                self.collider_set,
                self.impulse_joint_set,
                self.multibody_joint_set,
                self.ccd_solver,
                self.query_pipeline,
            )

            # print position of all balls
            return [
                (
                    self.rigid_body_set.get(self.balls[i]).translation().x(),
                    self.rigid_body_set.get(self.balls[i]).translation().y(),
                )
                for i in range(n_balls)
            ]


# Define the experiment
def my_experiment(wm):
    # set-up the simulation
    sim = BubbleSimulation()

    # run the simulation for a few steps
    for i in range(n_init_steps):
        next(sim)

    # create a window
    window = wm.create_default_window()

    # receive keyboard input from the window
    kb = window.create_event_receiver()

    resources = os.path.join(os.path.dirname(__file__), "resources")

    ball_stims = []

    for i in range(n_balls):

        stim = GaborStimulus(
            window,
            Circle(Pixels(0), Pixels(0), ScreenWidth(0.05)),
            0,
            Pixels(20),
            ScreenWidth(0.01),
            ScreenWidth(0.01),
            0,
            (0.0, 0.0, 0.0),
        )

        ball_stims.append(stim)

    stim2 = ImageStimulus(
        window,
        Rectangle(Pixels(-50), Pixels(-50), Pixels(100), Pixels(100)),
        os.path.join(resources, "crosshair.png"),
    )

    stim3 = psy.SpriteStimulus(
        window,
        Rectangle(
            ScreenWidth(-0.05), ScreenWidth(-0.05), ScreenWidth(0.1), ScreenWidth(0.1)
        ),
        os.path.join(
            resources,
            "buble_pop_two_spritesheet_512px_by512px_per_frame.png",
        ),
        4,
        2,
        fps=20,
        repeat=1,
    )

    # sleep for 1s
    time.sleep(1)
    subject_id = wm.prompt("Press any key to start the experiment")
    print(f"Subject ID: {subject_id}")

    stim3.reset()

    last_mouse_pos = (Pixels(0), Pixels(0))

    while True:
        for i in range(100):
            # stim1.set_orientation(stim1.orientation() + 0.01)
            frame = window.get_frame()
            frame.set_bg_color((0.5, 0.5, 0.5))
            # advance the simulation
            new_pos = next(sim)

            for i, stim in enumerate(ball_stims):
                # add the stimulus to the frame
                frame.add(stim)

                # move the stimulus
                stim.set_translation(Pixels(new_pos[i][0]), Pixels(new_pos[i][1]))

            frame.add(stim2)
            frame.add(stim3)

            window.submit_frame(frame)

            # check for new events
            events = kb.events()
            for i in range(len(events)):
                event = events[i]
                data = event.data

                if isinstance(data, psy.EventData.CursorMoved):

                    # update the position of the image stimulus
                    stim2.set_translation(data.position[0], data.position[1])

                    # update the position of the mouse
                    last_mouse_pos = data.position

                if isinstance(data, psy.EventData.MouseButtonPress):

                    # remove the last stimulus from ball_stims
                    ball_stims.pop() if len(ball_stims) > 0 else None

                    # move stimulus 3 to the position of the mouse
                    stim3.set_translation(last_mouse_pos[0], last_mouse_pos[1])

                    # reset the sprite
                    stim3.reset()

                if isinstance(data, psy.EventData.KeyPress):
                    if data.key == "n" and len(ball_stims) < n_balls:

                        # add a new stimulus to ball_stims
                        stim = GaborStimulus(
                            window,
                            Circle(Pixels(0), Pixels(0), ScreenWidth(0.05)),
                            0,
                            Pixels(20),
                            ScreenWidth(0.01),
                            ScreenWidth(0.01),
                            0,
                            (0.0, 0.0, 0.0),
                        )

                        ball_stims.append(stim)


if __name__ == "__main__":

    # Set the logging level
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"

    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.INFO)

    # Create an experiment manager
    em = ExperimentManager()

    # Get a monitor (0 is usually the internal screen, 1 the first external monitor, etc.)
    monitor = em.get_available_monitors()[-1]

    # Run the experiment
    em.run_experiment(my_experiment)
