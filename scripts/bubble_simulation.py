import rapier2d_py
import numpy as np

# def correct_velocity(velocity, speed):
#     np_velocity = np.array([velocity.x(), velocity.y()])
#     np_dir = np_velocity / np.linalg.norm(np_velocity)
#     np_velocity = np_dir * speed
#     return rapier2d_py.RealVector(np_velocity[0], np_velocity[1])


# create an Iterator that simulates the movement of a stimulus
class BubbleSimulation:

    def __init__(self, bubble_radius, n_bubbles=0, area_width=1000, area_height=1000, n_init_steps=0):
        # set up the simulation
        self.area_width = area_width
        self.area_height = area_height
        self.speed = 100.0
        self.n_bubbles = n_bubbles
        self.bubble_radius = bubble_radius

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

        # run the simulation for n_init_steps (call next() n_init_steps times)
        for _ in range(n_init_steps):
            next(self)

    def remove_bubble(self, handle):
        rigid_body_handle, collider_handle = handle
        #Error calling experiment_fn: PyErr { type: <class 'TypeError'>, value: TypeError("ColliderSet.remove() missing 3 required positional arguments: 'islands', 'bodies', and 'wake_up'"), traceback: Some(<traceback object at 0x108aa3a40>) }
        self.collider_set.remove(collider_handle, self.island_manager, self.rigid_body_set, True)
        self.handles.remove(handle)

    def add_bubble(self):
        # to place a bubble in an appropriate position, we draw a random position
        # and check if it is not too close to any other bubble


        initial_position = rapier2d_py.RealVector(0.0, 0.0)
        while True:
            # draw a random position
            initial_position = rapier2d_py.RealVector(
                np.random.uniform(-self.area_width / 2, self.area_width / 2),
                np.random.uniform(-self.area_height / 2, self.area_height / 2),
            )

            # check if it is too close to any other bubble
            too_close = False
            # for bubble in self.bubbles:
            #     if (initial_position - self.rigid_body_set.get(bubble).translation()).norm() < 200:
            #         too_close = True
            #         break

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

        # return the index of the new bubble
        return handle

    def __iter__(self):
        return self

    def __next__(self):


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
            return {handle:

                (
                self.rigid_body_set.get(self.handles[i][0]).translation().x(),
                self.rigid_body_set.get(self.handles[i][0]).translation().y(),
            ) for i, handle in enumerate(self.handles)}
