import rapier2d_py

# create sets
collider_set = rapier2d_py.ColliderSet()
rigid_body_set = rapier2d_py.RigidBodySet()


# create walls
walls_vertices = [
    rapier2d_py.Point(0.0, 0.0),
    rapier2d_py.Point(100.0, 0.0),
    rapier2d_py.Point(100.0, 100.0),
    rapier2d_py.Point(0.0, 100.0),
    rapier2d_py.Point(0.0, 0.0),
]

wall_collider = rapier2d_py.Collider(
    collider_type="polyline",
    vertices=walls_vertices,
    restitution=1.0,  # no energy loss
    friction=0.0,  # no friction
)

collider_set.insert(wall_collider)

n_balls = 1

balls = []

for i in range(n_balls):
    # build a collider
    ball = rapier2d_py.Collider(
        collider_type="ball",
        radius=1.0,
        restitution=1.0,  # no energy loss
        friction=0.0,  # no friction
    )

    # build a rigid body
    ball_rigid_body = rapier2d_py.RigidBody(
        body_type="dynamic",
        translation=rapier2d_py.Vector(10.0, 50.0),
    )
    # add rigid body to rigid body set
    rigid_body_handle = rigid_body_set.insert(ball_rigid_body)
    collider_set.insert_with_parent(ball, rigid_body_handle, rigid_body_set)

    balls.append(rigid_body_handle)

# set-up physics
gravity = rapier2d_py.Vector(0.0, 10.0)
integration_parameters = rapier2d_py.IntegrationParameters.default()
physics_pipeline = rapier2d_py.PhysicsPipeline()
island_manager = rapier2d_py.IslandManager()
broad_phase = rapier2d_py.DefaultBroadPhase()
narrow_phase = rapier2d_py.NarrowPhase()
impulse_joint_set = rapier2d_py.ImpulseJointSet()
multibody_joint_set = rapier2d_py.MultibodyJointSet()
ccd_solver = rapier2d_py.CCDSolver()
query_pipeline = rapier2d_py.QueryPipeline()


# run physics
for i in range(100):
    physics_pipeline.step(
        gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        query_pipeline,
    )
    # print position of first ball
    print(rigid_body_set.get(balls[0]).translation().y())
