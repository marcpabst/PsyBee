import rapier2d_py

# create a rect
rect = [
    rapier2d_py.Point(0.0, 0.0),
    rapier2d_py.Point(1.0, 0.0),
    rapier2d_py.Point(1.0, 1.0),
    rapier2d_py.Point(0.0, 1.0),
    rapier2d_py.Point(0.0, 0.0),
]

# create collider set
collider_set = rapier2d_py.ColliderSet()
rigid_body_set = rapier2d_py.RigidBodySet()

# build a collider
collider = rapier2d_py.Collider(
    collider_type="polyline",
    vertices=rect,
    restitution=1.0,  # no energy loss
    friction=0.0,  # no friction
)

# add collider to collider set
collider_set.insert(collider)

# build a rigid body
rigid_body = rapier2d_py.RigidBody(
    body_type="dynamic",
)

# add rigid body to rigid body set
rigid_body_handle = rigid_body_set.insert(rigid_body)


# add rigid body to collider set
collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set)

# //         let gravity = vector![0.0, 0.0];
# //         let mut integration_parameters = IntegrationParameters::default();
# //         integration_parameters.dt = 1.0 / MONITOR_HZ as f32;
# //         let physics_pipeline = PhysicsPipeline::new();
# //         let island_manager = IslandManager::new();
# //         let broad_phase = BroadPhase::new();
# //         let narrow_phase = NarrowPhase::new();
# //         let impulse_joint_set = ImpulseJointSet::new();
# //         let multibody_joint_set = MultibodyJointSet::new();
# //         let ccd_solver = CCDSolver::new();
# //         let query_pipeline = QueryPipeline::new();

gravity = rapier2d_py.Vector(0.0, 0.0)
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
for i in range(1000):
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

    # print step
    print(f"Step {i}")


print(collider_set)
