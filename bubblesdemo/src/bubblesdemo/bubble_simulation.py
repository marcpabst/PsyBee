import rapier2d_py
from rapier2d_py import RealVector
import numpy as np
import random
import time
from psydk import GaborStimulus

# def correct_velocity(velocity, speed):
#     np_velocity = np.array([velocity.x(), velocity.y()])
#     np_dir = np_velocity / np.linalg.norm(np_velocity)
#     np_velocity = np_dir * speed
#     return rapier2d_py.RealVector(np_velocity[0], np_velocity[1])


# create an Iterator that simulates the movement of a stimulus
class BubbleSimulation:

    def __init__(self, bubble_radius, area_width=1000, area_height=1000, n_init_steps=0,
        n_bubbles = 1, # number of bubbles visible on the screen at the same time
        duration_bubbles = 8.0, # max. duration in seconds a bubble is visible on the screen
        duration_bubbles_jitter = 0.0, # uniform jitter in seconds for the duration of the bubbles
        duration_delay = 3.0, # duration in seconds until the next bubble
        duration_delay_jitter = 2.0, # uniform jitter in seconds for the delay until the next bubble
    ):
        # set up the simulation
        self.area_width = area_width
        self.area_height = area_height
        self.speed = 100.0
        self.n_bubbles = n_bubbles
        self.bubble_radius = bubble_radius
        self.duration_bubbles = duration_bubbles
        self.duration_bubbles_jitter = duration_bubbles_jitter
        self.duration_delay = duration_delay
        self.duration_delay_jitter = duration_delay_jitter
        self.is_running = False


        # create stimulus list
        self.stimuli = []

        # create sets
        self.collider_set = rapier2d_py.ColliderSet()
        self.rigid_body_set = rapier2d_py.RigidBodySet()

        # create walls for a FHD screen, with (0,0) at the center
        walls_vertices = [
            rapier2d_py.RealPoint(-area_width / 2, -area_height / 2),
            rapier2d_py.RealPoint(area_width / 2, -area_height / 2),
            rapier2d_py.RealPoint(area_width / 2, area_height / 2),
            rapier2d_py.RealPoint(-area_width / 2, area_height / 2),
            rapier2d_py.RealPoint(-area_width / 2, -area_height / 2),
        ]

        wall_collider = rapier2d_py.Collider(
            collider_type="polyline",
            vertices=walls_vertices,
            restitution=1.0,  # no energy loss
            friction=0.0,  # no friction
        )

        self.collider_set.insert(wall_collider)

        self.handles = []

        for i in range(self.n_bubbles):
            self.add_bubble();

        # set-up physics
        self.gravity = RealVector(0.0, 0.0)
        self.integration_parameters = rapier2d_py.IntegrationParameters.default()
        self.physics_pipeline = rapier2d_py.PhysicsPipeline()
        self.island_manager = rapier2d_py.IslandManager()
        self.broad_phase = rapier2d_py.DefaultBroadPhase()
        self.narrow_phase = rapier2d_py.NarrowPhase()
        self.impulse_joint_set = rapier2d_py.ImpulseJointSet()
        self.multibody_joint_set = rapier2d_py.MultibodyJointSet()
        self.ccd_solver = rapier2d_py.CCDSolver()
        self.query_pipeline = rapier2d_py.QueryPipeline()

        # run the simulation for n_init_steps (call next() n_init_steps times)
        for _ in range(n_init_steps):
            next(self)

    def run(self):
        self.is_running = True

    def remove_bubble(self, handle):
        rigid_body_handle, collider_handle = handle
        self.collider_set.remove(collider_handle, self.island_manager, self.rigid_body_set, True)
        self.handles.remove(handle)

    def get_stimuli(self):
        return [bubble["stim"] for bubble in self.stimuli]

    def check_position(self, pos, window):
        """ Check if the position is within an active bubble """

        for (i, bubble) in enumerate(self.stimuli):

            # check if the bubble was touched/clicked
            if bubble["stim"].contains(pos[0], pos[1], window) and bubble["stim"].visible():
                # if yes, remove the bubble
                bubble["stim"].hide()
                self.remove_bubble(bubble["handle"])
                bubble["t_end"] = time.time()

                return (bubble["stim"]["cx"], bubble["stim"]["cy"])

        return False


    def add_bubble(self):
        # to place a bubble in an appropriate position, we draw a random position
        # and check if it is not too close to any other bubble
        initial_position = RealVector(0.0, 0.0)
        while True:
            # draw a random position
            initial_position = RealVector(
                np.random.uniform(-self.area_width / 2, self.area_width / 2),
                np.random.uniform(-self.area_height / 2, self.area_height / 2),
            )


            # check if it is too close to any other bubble
            too_close = False

            if not too_close:
                break


        # build a collider
        bubble = rapier2d_py.Collider(
            collider_type="ball",
            radius=self.bubble_radius,
            restitution=1.0,  # no energy loss
            friction=0.0,  # no friction
        )

        # calculate the initial position of the ball (in a grid)
        max_balls_per_row = int(np.sqrt(self.n_bubbles))

        # create a random direction in radians
        direction = np.random.uniform(0, 2 * np.pi)

        # create a random velocity vector
        linvel = rapier2d_py.RealVector(self.speed * np.cos(direction), self.speed * np.sin(direction))

        # build a rigid body
        ball_rigid_body = rapier2d_py.RigidBody(
            body_type="dynamic",
            translation=initial_position,
            linvel=linvel,
        )



        # add rigid body to rigid body set
        rigid_body_handle = self.rigid_body_set.insert(ball_rigid_body)
        collider_handle = self.collider_set.insert_with_parent(bubble, rigid_body_handle, self.rigid_body_set)

        handle = (rigid_body_handle, collider_handle)

        self.handles.append(handle)



        stim = GaborStimulus(
                    0, 0,  # center x, y
                    "3cm", # size
                    ".5cm",# cycle length
                    "2cm", # sigma
        )

        t_start = time.time()
        dur = self.duration_bubbles + np.random.uniform(-self.duration_bubbles_jitter, self.duration_bubbles_jitter)
        dle = self.duration_delay + np.random.uniform(self.duration_delay_jitter, self.duration_delay_jitter)

        self.stimuli.append({ "stim": stim, "t_start": t_start, "duration": dur, "handle": handle, "delay": dle})

    def __iter__(self):
        return self

    def __next__(self):
        """ Run the physics simulation for one step and then return a list of drawable stimuli """
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

            # update the stimuli
            t = time.time()
            stimuli = []

            for j, s in enumerate(list(self.stimuli)):

                s["stim"]["cx"] = self.rigid_body_set.get(s["handle"][0]).translation().x()
                s["stim"]["cy"] = self.rigid_body_set.get(s["handle"][0]).translation().y()

                stimuli.append(s["stim"])

                # check if the bubble has timed out
                if s["stim"].visible() and time.time() - s["t_start"] > s["duration"]:
                    # hide the stimulus
                    s["stim"].hide()
                    # remove the bubble from simulation
                    print("remove bubble with handle", s["handle"])
                    self.remove_bubble(s["handle"])
                    # set the end time
                    s["t_end"] = time.time()

                # check if the bubble should be removed
                if "t_end" in s:
                    if time.time() - s["t_end"] > s["delay"]:

                        # remove the bubble from the list
                        print("remove bubble no", j)
                        del self.stimuli[j]

                        self.add_bubble()

                # if t - s["t_start"] > s["delay"]:
                #     stimuli.append(s["stim"])
                #     if t - s["t_start"] > s["delay"] + s["duration"]:
                #         self.remove_bubble(s["handle"])
                #         self.stimuli.remove(s)


            return stimuli
