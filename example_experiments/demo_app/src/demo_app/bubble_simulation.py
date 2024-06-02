# Copyright (c) 2024 Marc Pabst
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import rapier2d_py
import numpy as np

# def correct_velocity(velocity, speed):
#     np_velocity = np.array([velocity.x(), velocity.y()])
#     np_dir = np_velocity / np.linalg.norm(np_velocity)
#     np_velocity = np_dir * speed
#     return rapier2d_py.RealVector(np_velocity[0], np_velocity[1])


# create an Iterator that simulates the movement of a stimulus
class BubbleSimulation:

    def __init__(self, n_balls=4, width=1920, height=1080, n_init_steps=0):
        # set up the simulation
        self.speed = 0.0
        self.n_balls = n_balls

        # create sets
        self.collider_set = rapier2d_py.ColliderSet()
        self.rigid_body_set = rapier2d_py.RigidBodySet()

        # create walls for a FHD screen, with (0,0) at the center
        walls_vertices = [
            rapier2d_py.RealPoint(-width / 2, -height / 2),
            rapier2d_py.RealPoint(width / 2, -height / 2),
            rapier2d_py.RealPoint(width / 2, height / 2),
            rapier2d_py.RealPoint(-width / 2, height / 2),
            rapier2d_py.RealPoint(-width / 2, -height / 2),
        ]

        wall_collider = rapier2d_py.Collider(
            collider_type="polyline",
            vertices=walls_vertices,
            restitution=1.0,  # no energy loss
            friction=0.0,  # no friction
        )

        self.collider_set.insert(wall_collider)

        self.balls = []

        for i in range(self.n_balls):
            # build a collider
            ball = rapier2d_py.Collider(
                collider_type="ball",
                radius=200.0,
                restitution=1.0,  # no energy loss
                friction=0.0,  # no friction
            )

            # calculate the initial position of the ball (in a grid)
            max_balls_per_row = int(np.sqrt(self.n_balls))
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
            self.collider_set.insert_with_parent(ball, rigid_body_handle, self.rigid_body_set)

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

        # run the simulation for n_init_steps (call next() n_init_steps times)
        for _ in range(n_init_steps):
            next(self)

    def __iter__(self):
        return self

    def __next__(self):

        # set velocity
        for i in range(self.n_balls):
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
            return [(
                self.rigid_body_set.get(self.balls[i]).translation().x(),
                self.rigid_body_set.get(self.balls[i]).translation().y(),
            ) for i in range(self.n_balls)]
