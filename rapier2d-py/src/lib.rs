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
use rapier2d::prelude::*;

// macro that wrap a struct into Py<Struct>(struct)
macro_rules! wrap {
    ($name:ident) => {
        paste::paste! {
            #[pyclass(name = "Py" $name)]
            pub struct [<Py $name>](pub $name);
        }
    };
}

macro_rules! forward_method {
    ($name:ident, $method:ident, $args:tt, $ret:tt) => {
        paste::paste! {
            #[pymethods]
            impl [<Py $name>] {
                pub fn [< $method >] $args -> $ret {
                    self.0.$method $args
                }
            }
        }
    };
}

wrap!(Collider);

wrap!(ColliderBuilder);

impl PyColliderBuilder {
    pub fn build(&self) -> Collider {
        self.0.build()
    }
}

wrap!(IntegrationParameters);
wrap!(RigidBodyBuilder);
wrap!(RigidBodySet);
wrap!(ColliderSet);
wrap!(PhysicsPipeline);
wrap!(IslandManager);
wrap!(NarrowPhase);
wrap!(ImpulseJointSet);
wrap!(MultibodyJointSet);
wrap!(CCDSolver);
wrap!(QueryPipeline);
