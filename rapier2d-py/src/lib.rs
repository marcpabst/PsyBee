use std::{borrow::Borrow, ops::Deref};

use derive_more::Display;
use paste::paste;
use pyo3::{prelude::*, types::PyList};
use pywrap::{py_forward, py_getter, py_wrap, transmute_ignore_size};
use rapier2d::{parry::shape, prelude::*};

// and for references (if type supports cloning)

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
type RealPoint = Point<Real>;

#[pyclass(name = "RealPoint")]
#[derive(Clone, Display)]
pub struct PyRealPoint(pub Point<Real>);

#[pymethods]
impl PyRealPoint {
    #[new]
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point::new(x, y))
    }
}

// Collider
py_wrap!(Collider);
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
        vertices: Option<Vec<PyRealPoint>>,
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
                let vertices = vertices.expect("Vertices are required").iter().map(|v| v.0).collect();
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
        builder = option_call!(builder, contact_force_event_threshold, contact_force_event_threshold);
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
py_wrap!(ColliderSet);
py_forward!(ColliderSet, fn new() -> Self);
py_forward!(ColliderSet, fn insert(&mut self, collider: Collider) -> ColliderHandle);
py_forward!(ColliderSet, fn insert_with_parent(&mut self, collider: Collider, parent_handle: RigidBodyHandle, bodies: &mut RigidBodySet) -> ColliderHandle);
py_forward!(ColliderSet, fn remove(&mut self, handle: ColliderHandle, islands: &mut IslandManager, bodies: &mut RigidBodySet, wake_up: bool) -> Option<Collider>);

// ColliderHandle
py_wrap!(ColliderHandle);
impl_clone_wrap!(PyColliderHandle);

// RigidBodySet
py_wrap!(RigidBodySet);
py_forward!(RigidBodySet, fn new() -> Self);
py_forward!(RigidBodySet, fn insert(&mut self, body: RigidBody) -> RigidBodyHandle);
py_forward!(RigidBodySet, fn get(&self, handle: RigidBodyHandle) -> Option<RigidBody>);

// RigidBodyHandle
py_wrap!(RigidBodyHandle);
impl_clone_wrap!(PyRigidBodyHandle);

// Vector
type RealVector = Vector<Real>;
py_wrap!(RealVector);
impl_clone_wrap!(PyRealVector);

#[pymethods]
impl PyRealVector {
    #[new]
    fn new(x: f32, y: f32) -> Self {
        Self(Vector::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }

    pub fn y(&self) -> f32 {
        self.0.y
    }
}

// AngVector
type RealAngVector = Vector<Real>;
py_wrap!(RealAngVector);

// Isometry
type RealIsometry = Isometry<Real>;
py_wrap!(RealIsometry);
impl_clone_wrap!(PyRealIsometry);

// RigidBody
py_wrap!(RigidBody);
py_forward!(RigidBody, fn translation(&self) -> RealVector);
py_forward!(RigidBody, fn linvel(&self) -> RealVector);
py_forward!(RigidBody, fn set_linvel(&mut self, linvel: &RealVector, wake_up: bool) -> ());
impl_clone_wrap!(PyRigidBody);

// SpringJoint
py_wrap!(SpringJoint);
py_forward!(SpringJoint, fn new(reest_length: f32, stiffness: f32, damping: f32) -> Self);

// #[pymethods]
// impl PyRigidBody {
//     fn translation(&mut self) -> PyRealVector {
//         PyRealVector(self.0.translation())
//     }
// }

// IntegrationParameters
py_wrap!(IntegrationParameters);
impl_clone_wrap!(PyIntegrationParameters);
forward_nullary_function!(IntegrationParameters, default -> Self);

#[pymethods]
impl PyIntegrationParameters {
    #[staticmethod]
    fn default2() -> PyIntegrationParameters {
        let mut i = IntegrationParameters::default();
        i.dt = 1.0 / 60.0;
        PyIntegrationParameters(i)
    }
}

// PhysicsPipeline
py_wrap!(PhysicsPipeline);

py_forward!(PhysicsPipeline, fn new() -> Self);

// IslandManager
py_wrap!(IslandManager);
impl_clone_wrap!(PyIslandManager);
py_forward!(IslandManager, fn new() -> Self);

// DefaultBroadPhase
py_wrap!(DefaultBroadPhase);
impl_clone_wrap!(PyDefaultBroadPhase);
py_forward!(DefaultBroadPhase, fn new() -> Self);

// NarrowPhase
py_wrap!(NarrowPhase);
impl_clone_wrap!(PyNarrowPhase);
py_forward!(NarrowPhase, fn new() -> Self);

// PyMultibodyJointHandle
py_wrap!(MultibodyJointHandle);

// ImpulseJointSet
py_wrap!(ImpulseJointSet);
impl_clone_wrap!(PyImpulseJointSet);
py_forward!(ImpulseJointSet, fn new() -> Self);

// MultibodyJointSet
py_wrap!(MultibodyJointSet);
impl_clone_wrap!(PyMultibodyJointSet);
py_forward!(MultibodyJointSet, fn new() -> Self);

#[pymethods]
impl PyMultibodyJointSet {
    pub fn insert_spring(
        &mut self,
        body1: PyRigidBodyHandle,
        body2: PyRigidBodyHandle,
        data: &PySpringJoint,
        wake_up: bool,
    ) -> Option<PyMultibodyJointHandle> {
        let data = data.0;
        let out = self.0.insert(body1.0, body2.0, data, wake_up);
        match out {
            Some(handle) => Some(PyMultibodyJointHandle(handle)),
            None => None,
        }
    }
}

// CCDSolver
py_wrap!(CCDSolver);
impl_clone_wrap!(PyCCDSolver);
py_forward!(CCDSolver, fn new() -> Self);

// QueryPipeline
py_wrap!(QueryPipeline);

impl_clone_wrap!(PyQueryPipeline);
py_forward!(QueryPipeline, fn new() -> Self);

#[pymethods]
impl PyRigidBody {
    #[new]
    pub fn new(
        body_type: &str,
        position: Option<PyRealIsometry>,
        translation: Option<PyRealVector>,
        linvel: Option<PyRealVector>,
    ) -> Self {
        let body_type = match body_type {
            "dynamic" => RigidBodyType::Dynamic,
            "kinematic_position_based" => RigidBodyType::KinematicPositionBased,
            "kinematic_velocity_based" => RigidBodyType::KinematicVelocityBased,
            "fixed" => RigidBodyType::Fixed,
            _ => panic!("Invalid body type"),
        };

        let mut builder = RigidBodyBuilder::new(body_type);

        builder = option_call!(builder, position, position);
        builder = option_call!(builder, translation, translation);
        builder = option_call!(builder, linvel, linvel);

        Self(builder.build())
    }
}

#[pymethods]
impl PyPhysicsPipeline {
    pub fn step(
        &mut self,
        gravity: &PyRealVector,
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
fn rapier2d_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCollider>()?;
    m.add_class::<PyRealPoint>()?;
    m.add_class::<PyColliderSet>()?;
    m.add_class::<PyColliderHandle>()?;
    m.add_class::<PyRigidBody>()?;
    m.add_class::<PyRealVector>()?;
    m.add_class::<PyRealAngVector>()?;
    m.add_class::<PyRealIsometry>()?;
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
    m.add_class::<PySpringJoint>()?;

    Ok(())
}
