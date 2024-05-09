// /// A balloon
// #[derive(Debug, Clone)]
// pub struct Balloon<T: Stimulus> {
//     pub position: Vector2<Real>,
//     pub radius: Real,
//     pub velocity: Vector2<Real>,
//     pub hidden: bool,
//     pub stimulus: T,
// }

// impl<T: Stimulus> BalloonSimulator<T> {
//     /// Create a new balloon simulator
//     pub fn new(
//         balloons: Vec<Balloon<T>>,
//         origin: Vector2<Real>,
//         extend: Vector2<Real>,
//     ) -> Self {
//         // all the things that can collide with each other will live in the collider set
//         let mut collider_set = ColliderSet::new();

//         // create the walls (they usually align with the window size)
//         let walls = ColliderBuilder::polyline(
//             vec![
//                 Point::new(origin.x, origin.y),            // bottom left
//                 Point::new(origin.x, origin.y + extend.y), // top left
//                 Point::new(origin.x + extend.x, origin.y + extend.y), // top right
//                 Point::new(origin.x + extend.x, origin.y), // bottom right
//                 Point::new(origin.x, origin.y),            // bottom left
//             ],
//             None,
//         )
//         .restitution(1.0) // no energy loss
//         .friction(0.0) // no friction
//         .build();

//         collider_set.insert(walls);

//         // create the balloons as rigid bodies and add them to the collider set
//         let mut balloon_set = RigidBodySet::new();
//         let mut balloon_handles = Vec::new();

//         // for each balloon, create a rigid body and a collider with the same size
//         for balloon in balloons.iter() {
//             let rigid_body = RigidBodyBuilder::dynamic()
//                 .translation(balloon.position)
//                 .linvel(balloon.velocity)
//                 .build();

//             let collider = ColliderBuilder::ball(balloon.radius)
//                 .restitution(1.0)
//                 .friction(0.0)
//                 .build();

//             let ball_body_handle = balloon_set.insert(rigid_body);
//             balloon_handles.push(ball_body_handle);

//             // insert the collider with the rigid body as parent
//             collider_set.insert_with_parent(collider, ball_body_handle, &mut balloon_set);
//         }

//         // create the physics pipeline
//         let gravity = vector![0.0, 0.0];
//         let mut integration_parameters = IntegrationParameters::default();
//         integration_parameters.dt = 1.0 / MONITOR_HZ as f32;
//         let physics_pipeline = PhysicsPipeline::new();
//         let island_manager = IslandManager::new();
//         let broad_phase = BroadPhase::new();
//         let narrow_phase = NarrowPhase::new();
//         let impulse_joint_set = ImpulseJointSet::new();
//         let multibody_joint_set = MultibodyJointSet::new();
//         let ccd_solver = CCDSolver::new();
//         let query_pipeline = QueryPipeline::new();

//         Self {
//             balloons,
//             collider_set,
//             balloon_set,
//             balloon_handles,
//             gravity,
//             integration_parameters,
//             physics_pipeline,
//             island_manager,
//             broad_phase,
//             narrow_phase,
//             impulse_joint_set,
//             multibody_joint_set,
//             ccd_solver,
//             query_pipeline,
//             n_steps: 1,
//         }
//     }
// }

// // implement the Iterator trait for the BalloonSimulator
// // each iteration steps through n_steps of the simulation
// impl<T: Stimulus + Clone> Iterator for BalloonSimulator<T> {
//     type Item = Vec<Balloon<T>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         for _ in 0..self.n_steps {
//             self.physics_pipeline.step(
//                 &self.gravity,
//                 &self.integration_parameters,
//                 &mut self.island_manager,
//                 &mut self.broad_phase,
//                 &mut self.narrow_phase,
//                 &mut self.balloon_set,
//                 &mut self.collider_set,
//                 &mut self.impulse_joint_set,
//                 &mut self.multibody_joint_set,
//                 &mut self.ccd_solver,
//                 Some(&mut self.query_pipeline),
//                 &(),
//                 &(),
//             );
//         }

//         // update the balloons
//         for (i, handle) in self.balloon_handles.iter().enumerate() {
//             let balloon = &mut self.balloons[i];
//             let rb = self.balloon_set.get(*handle).unwrap();
//             balloon.position = rb.position().translation.vector;
//             balloon.velocity = rb.linvel().clone();
//         }
//         Some(self.balloons.clone())
//     }
// }

use paste::paste;
use pyo3::prelude::*;

use pyo3::types::PyList;
use rapier2d::{parry::shape, prelude::*};

// macro that wrap a struct into Py<Struct>(struct)
macro_rules! wrap {
    ($name:ident) => {


        paste::paste! {

             // the wrapper struct
            #[pyclass(name = "" $name)]
            pub struct [<Py $name>](pub $name);

            // Into trait implementation
            impl Into<$name> for [<Py $name>] {
                fn into(self) -> $name {
                    self.0
                }
            }

        }


    };
    ($name:ident<$($t:tt),*>) => {
        paste::paste! {

             // the wrapper struct
            #[pyclass(name = "" $name)]
            pub struct [<Py $name>](pub $name<$($t),*>);

            // Into trait implementation
            impl Into<$name<$($t),*>> for [<Py $name>] {
                fn into(self) -> $name<$($t),*> {
                    self.0
                }
            }
        }
    };
}

macro_rules! forward_nullary_new {
    ($name:ident) => {
        paste::paste! {
            #[pymethods]
            impl [<Py $name>] {
                #[new]
                pub fn new() -> Self {
                    Self($name::new())
                }
            }
        }
    };
}

macro_rules! forward_nullary_function {
    ($name:ident, $method:ident -> $ret:tt) => {
        paste::paste! {
            #[pymethods]
            impl [<Py $name>] {
                #[staticmethod]
                pub fn [< $method >]() -> $ret {
                    $ret($name::$method())
                }
            }
        }
    };
}

macro_rules! forward_method {
    ($name:ident, $method:ident($arg:ident : $argtype:tt) -> $ret:tt) => {
        paste::paste! {
            #[pymethods]
            impl [<Py $name>] {
                pub fn [< $method >](&self, $arg : [< $argtype >]) -> $ret {
                    $ret(self.0.$method($arg.into()))
                }
            }
        }
    };
}

macro_rules! forward_staticmethod {
    ($name:ident, $method:ident($arg:ident : $argtype:tt) -> $ret:tt) => {
        paste::paste! {
            #[pymethods]
            impl [<Py $name>] {
                #[staticmethod]
                pub fn [< $method >]($arg : [< $argtype >]) -> $ret {
                    $ret($name::$method($arg))
                }
            }
        }
    };
}

macro_rules! option_call {
    ($func:ident, $option:expr) => {
        match $option {
            Some(val) => {
                $func(val);
            }

            None => (),
        };
    };
    ($s:expr, $func:ident, $option:expr) => {
        match $option {
            Some(val) => $s.$func(val.into()),
            None => $s,
        };
    };
}

macro_rules! impl_clone_wrap {
    ($name:ident) => {
        impl Clone for $name {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
    };
}

// Point
wrap!(Point<Real>);
impl_clone_wrap!(PyPoint);

#[pymethods]
impl PyPoint {
    #[new]
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point::new(x, y))
    }
}

// Collider
wrap!(Collider);
impl_clone_wrap!(PyCollider);

#[pymethods]
impl PyCollider {
    #[new]
    pub fn new(
        collider_type: &str,

        // options for different collider shapes
        half_height: Option<f32>,
        radius: Option<f32>,
        border_radius: Option<f32>,
        vertices: Option<Vec<PyPoint>>,
        indices: Option<Vec<[u32; 2]>>,

        // options for all colliders
        user_data: Option<u128>,
        // collision_groups: Option<InteractionGroups>,
        // solver_groups: Option<InteractionGroups>,
        is_sensor: Option<bool>,
        // active_hooks: Option<ActiveHooks>,
        // active_events: Option<ActiveEvents>,
        // active_collision_types: Option<ActiveCollisionTypes>,
        friction: Option<f32>,
        // friction_combine_rule: Option<CoefficientCombineRule>,
        restitution: Option<f32>,
        // restitution_combine_rule: Option<CoefficientCombineRule>,
        density: Option<f32>,
        mass: Option<f32>,
        // mass_properties: Option<MassProperties>,
        contact_force_event_threshold: Option<f32>,
        // translation: Option<Vector<Real>>,
        rotation: Option<AngVector<Real>>,
        // position: Option<Isometry<Real>>,
        // position_wrt_parent: Option<Isometry<Real>>,
        // delta: Option<Isometry<Real>>,
        contact_skin: Option<Real>,
        enabled: Option<bool>,
    ) -> Self {
        let mut builder: ColliderBuilder = match collider_type {
            "ball" => ColliderBuilder::ball(radius.expect("Radius is required")),
            "polyline" => {
                let vertices = vertices
                    .expect("Vertices are required")
                    .into_iter()
                    .map(|p| p.0)
                    .collect();
                ColliderBuilder::polyline(vertices, indices)
            }
            _ => panic!("Invalid collider type"),
        };

        builder = option_call!(builder, user_data, user_data);
        // builder = option_call!(builder, collision_groups, collision_groups);
        // builder = option_call!(builder, solver_groups, solver_groups);
        builder = option_call!(builder, sensor, is_sensor);
        // builder = option_call!(builder, active_hooks, active_hooks);
        // builder = option_call!(builder, active_events, active_events);
        // builder = option_call!(builder, active_collision_types, active_collision_types);
        builder = option_call!(builder, friction, friction);
        // builder = option_call!(builder, friction_combine_rule, friction_combine_rule);
        builder = option_call!(builder, restitution, restitution);
        // builder = option_call!(builder, restitution_combine_rule, restitution_combine_rule);
        builder = option_call!(builder, density, density);
        builder = option_call!(builder, mass, mass);
        // builder = option_call!(builder, mass_properties, mass_properties);
        builder = option_call!(
            builder,
            contact_force_event_threshold,
            contact_force_event_threshold
        );
        // builder = option_call!(builder, translation, translation);
        builder = option_call!(builder, rotation, rotation);
        // builder = option_call!(builder, position, position);
        // builder = option_call!(builder, position_wrt_parent, position_wrt_parent);
        // builder = option_call!(builder, delta, delta);
        builder = option_call!(builder, contact_skin, contact_skin);
        builder = option_call!(builder, enabled, enabled);

        Self(builder.build())
    }
}

// ColliderSet
wrap!(ColliderSet);
forward_nullary_new!(ColliderSet);

#[pymethods]
impl PyColliderSet {
    pub fn insert(&mut self, collider: PyCollider) -> PyColliderHandle {
        PyColliderHandle(self.0.insert(collider.0))
    }

    pub fn insert_with_parent(
        &mut self,
        collider: PyCollider,
        parent_handle: PyRigidBodyHandle,
        bodies: &mut PyRigidBodySet,
    ) -> PyColliderHandle {
        PyColliderHandle(self.0.insert_with_parent(
            collider.0,
            parent_handle.0,
            &mut bodies.0,
        ))
    }
}
// ColliderHandle
wrap!(ColliderHandle);

// RigidBodySet
wrap!(RigidBodySet);
forward_nullary_new!(RigidBodySet);
impl_clone_wrap!(PyRigidBodySet);

#[pymethods]
impl PyRigidBodySet {
    pub fn insert(&mut self, body: PyRigidBody) -> PyRigidBodyHandle {
        PyRigidBodyHandle(self.0.insert(body.0))
    }
}

// RigidBodyHandle
wrap!(RigidBodyHandle);
impl_clone_wrap!(PyRigidBodyHandle);

// Vector
wrap!(Vector<Real>);
impl_clone_wrap!(PyVector);

#[pymethods]
impl PyVector {
    #[new]
    fn new(x: f32, y: f32) -> Self {
        Self(Vector::new(x, y))
    }
}

// AngVector
wrap!(AngVector<Real>);
impl_clone_wrap!(PyAngVector);

// Isometry
wrap!(Isometry<Real>);
impl_clone_wrap!(PyIsometry);

// RigidBody
wrap!(RigidBody);
impl_clone_wrap!(PyRigidBody);

// IntegrationParameters
wrap!(IntegrationParameters);
impl_clone_wrap!(PyIntegrationParameters);
forward_nullary_function!(IntegrationParameters, default -> Self);

// PhysicsPipeline
wrap!(PhysicsPipeline);
forward_nullary_new!(PhysicsPipeline);

// IslandManager
wrap!(IslandManager);
impl_clone_wrap!(PyIslandManager);
forward_nullary_new!(IslandManager);

// DefaultBroadPhase
wrap!(DefaultBroadPhase);
impl_clone_wrap!(PyDefaultBroadPhase);
forward_nullary_new!(DefaultBroadPhase);

// NarrowPhase
wrap!(NarrowPhase);
impl_clone_wrap!(PyNarrowPhase);
forward_nullary_new!(NarrowPhase);

// ImpulseJointSet
wrap!(ImpulseJointSet);
impl_clone_wrap!(PyImpulseJointSet);
forward_nullary_new!(ImpulseJointSet);

// MultibodyJointSet
wrap!(MultibodyJointSet);
impl_clone_wrap!(PyMultibodyJointSet);
forward_nullary_new!(MultibodyJointSet);

// CCDSolver
wrap!(CCDSolver);
impl_clone_wrap!(PyCCDSolver);
forward_nullary_new!(CCDSolver);

// QueryPipeline
wrap!(QueryPipeline);
impl_clone_wrap!(PyQueryPipeline);
forward_nullary_new!(QueryPipeline);

// RigidBodyBuilder

#[pymethods]
impl PyRigidBody {
    #[new]
    pub fn new(body_type: &str, position: Option<PyIsometry>) -> Self {
        let body_type = match body_type {
            "dynamic" => RigidBodyType::Dynamic,
            "kinematic_position_based" => RigidBodyType::KinematicPositionBased,
            "kinematic_velocity_based" => RigidBodyType::KinematicVelocityBased,
            "fixed" => RigidBodyType::Fixed,
            _ => panic!("Invalid body type"),
        };

        let mut builder = RigidBodyBuilder::new(body_type);

        builder = option_call!(builder, position, position);

        Self(builder.build())
    }
}

#[pymethods]
impl PyPhysicsPipeline {
    pub fn step(
        &mut self,
        gravity: PyVector,
        integration_parameters: PyIntegrationParameters,
        island_manager: &mut PyIslandManager,
        broad_phase: &mut PyDefaultBroadPhase,
        narrow_phase: &mut PyNarrowPhase,
        bodies: &mut PyRigidBodySet,
        colliders: &mut PyColliderSet,
        impulse_joint_set: &mut PyImpulseJointSet,
        multibody_joint_set: &mut PyMultibodyJointSet,
        ccd_solver: &mut PyCCDSolver,
        query_pipeline: Option<&mut PyQueryPipeline>,
    ) {
        let query_pipeline: Option<&mut QueryPipeline> = match query_pipeline {
            Some(qp) => Some(&mut qp.0),
            None => None,
        };

        self.0.step(
            &gravity.0,
            &integration_parameters.0,
            &mut island_manager.0,
            &mut broad_phase.0,
            &mut narrow_phase.0,
            &mut bodies.0,
            &mut colliders.0,
            &mut impulse_joint_set.0,
            &mut multibody_joint_set.0,
            &mut ccd_solver.0,
            query_pipeline,
            &(),
            &(),
        );
    }
}

#[pymodule]
fn rapier2d_py<'py, 'a>(
    _py: Python<'py>,
    m: &'a pyo3::prelude::PyModule,
) -> Result<(), pyo3::PyErr> {
    m.add_class::<PyCollider>()?;
    m.add_class::<PyPoint>()?;
    m.add_class::<PyColliderSet>()?;
    m.add_class::<PyColliderHandle>()?;
    m.add_class::<PyRigidBody>()?;
    m.add_class::<PyVector>()?;
    m.add_class::<PyAngVector>()?;
    m.add_class::<PyIsometry>()?;
    m.add_class::<PyRigidBodySet>()?;
    m.add_class::<PyRigidBodyHandle>()?;
    m.add_class::<PyIntegrationParameters>()?;
    m.add_class::<PyPhysicsPipeline>()?;
    m.add_class::<PyIslandManager>()?;
    m.add_class::<PyDefaultBroadPhase>()?;
    m.add_class::<PyNarrowPhase>()?;
    m.add_class::<PyImpulseJointSet>()?;
    m.add_class::<PyMultibodyJointSet>()?;
    m.add_class::<PyCCDSolver>()?;
    m.add_class::<PyQueryPipeline>()?;

    Ok(())
}

// wrap!(IntegrationParameters);
// wrap!(RigidBodyBuilder);
// wrap!(RigidBodySet);
// wrap!(ColliderSet);
// wrap!(PhysicsPipeline);
// wrap!(IslandManager);
// wrap!(NarrowPhase);
// wrap!(ImpulseJointSet);
// wrap!(MultibodyJointSet);
// wrap!(CCDSolver);
// wrap!(QueryPipeline);
