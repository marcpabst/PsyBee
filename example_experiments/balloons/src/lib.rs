use lazy_static::lazy_static;
use mobile_entry_point::mobile_entry_point;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
#[cfg(target_os = "ios")]
use objc::class;
#[cfg(target_os = "ios")]
use objc::rc::StrongPtr;
#[cfg(target_os = "ios")]
use objc::runtime::{Class, Object, Sel};
#[cfg(target_os = "ios")]
use objc::{declare::ClassDecl, msg_send, sel, sel_impl};
use psychophysics::visual::stimuli::patterns::Checkerboard;
use psychophysics::visual::stimuli::patterns::GaborPatch;
use psychophysics::visual::stimuli::patterns::Image;
use psychophysics::visual::stimuli::patterns::PolkaDots;
use psychophysics::visual::stimuli::patterns::Sprite;
use psychophysics::{
    prelude::*,
    visual::{
        geometry::Transformable,
        stimuli::{patterns::SineGratings, Stimulus},
    },
    ExperimentManager, WindowManager, WindowOptions,
};
use rand_distr::Distribution;
use rapier2d::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "ios")]
pub type id = *mut Object;

const N_BALLOONS: usize = 10;
const N_BALLOON_RADIUS: f32 = 150.0;
const SPEED: f32 = 100.0;

const MONITOR_HZ: f64 = 120.0;

// global variable to store current gaze position behind a mutex
lazy_static! {
    static ref GAZE_POSITION: Arc<Mutex<Option<Vector2<f32>>>> =
        Arc::new(Mutex::new(None));
}

#[cfg(target_os = "ios")]
#[repr(C)]
#[derive(Debug)]
struct MyMatrix {
    c1: [f32; 4],
    c2: [f32; 4],
    c3: [f32; 4],
    c4: [f32; 4],
}
#[cfg(target_os = "ios")]
// impl into nalgebra::Matrix4<f32> for SimdFloat4x4
impl Into<Matrix4<f32>> for MyMatrix {
    fn into(self) -> Matrix4<f32> {
        Matrix4::new(
            self.c1[0], self.c1[1], self.c1[2], self.c1[3], self.c2[0], self.c2[1],
            self.c2[2], self.c2[3], self.c3[0], self.c3[1], self.c3[2], self.c3[3],
            self.c4[0], self.c4[1], self.c4[2], self.c4[3],
        )
        .transpose()
    }
}
#[cfg(target_os = "ios")]
/// Compute the intersection of a line with the Z=0 plane
fn intersection_with_z_plane(
    point: Vector3<f32>,
    vector: Vector3<f32>,
) -> Option<Vector2<f32>> {
    if vector.z == 0.0 {
        // The line is parallel to the Z=0 plane (no intersection or the line lies in the plane)
        None
    } else {
        // Compute the intersection point
        let t = -point.z / vector.z;
        let x = point.x + t * vector.x;
        let y = point.y + t * vector.y;

        // Return the intersection point as a 2D vector
        Some(Vector2::new(x, y))
    }
}

#[cfg(target_os = "ios")]
fn get_current_gaze_position(frame: id) -> Option<nalgebra::Vector2<f32>> {
    // get the array of anchors using msg_send!
    let anchors: id = unsafe { msg_send![frame, anchors] };

    // check if length of anchors is larger than 0
    let num_anchors: usize = unsafe { msg_send![anchors, count] };

    println!("num_anchors: {:?}", num_anchors);

    if num_anchors == 0 {
        return None;
    }

    let i = 0;
    // get the anchor at index i
    let anchor: id = unsafe { msg_send![anchors, objectAtIndex: i] };

    // check if the anchor is a face anchor
    let is_face_anchor: bool =
        unsafe { msg_send![anchor, isMemberOfClass: class!(ARFaceAnchor)] };

    if is_face_anchor {
        // the matrix that transforms from face space to world space
        let raw_ptr: id = unsafe { msg_send![anchor, transform] };
        let face_transform: MyMatrix =
            unsafe { msg_send![class!(MatrixConverter), convertToMyMatrix:raw_ptr] };
        let face_transform: Matrix4<f32> = face_transform.into();

        // print first column of the matrix
        println!("face_transform c1: {:}", face_transform.column(0));

        // the matrix that transforms from right eye space to face space
        let raw_ptr: id = unsafe { msg_send![anchor, transform] };
        let right_eye_transform: MyMatrix =
            unsafe { msg_send![class!(MatrixConverter), convertToMyMatrix:raw_ptr] };
        let right_eye_transform: Matrix4<f32> = right_eye_transform.into();
        let right_eye_face_transform = face_transform * right_eye_transform;

        // create two points in eye space
        let p = Vector4::new(0.0, 0.0, 0.0, -1.0);
        let v = Vector4::new(0.0, 0.0, 1.0, 1.0);

        let right_eye_position = (right_eye_face_transform * p).xyz();
        let right_eye_vec = (right_eye_face_transform * v).xyz();

        // the matrix that transforms from right eye space to face space
        let raw_ptr: id = unsafe { msg_send![anchor, lookAtPoint] };
        let look_at_point_: [f32; 3] =
            unsafe { msg_send![class!(MatrixConverter), convertToMyVector:raw_ptr] };
        let look_at_point = face_transform
            * Vector4::new(look_at_point_[0], look_at_point_[1], look_at_point_[2], 1.0);

        let right_eye_distance = right_eye_position.z * 100.0;

        println!("right_eye_distance: {:?} cm", right_eye_distance);
        println!("look_at_point: {:?}", look_at_point_);
        println!("right_eye_vec: {:?}", right_eye_vec);

        // // compute intersection of the gaze vector with the screen (z=0)
        // intersection_with_z_plane(right_eye_position, right_eye_vec)

        Some(Vector2::new(look_at_point.x, look_at_point.y))
    } else {
        return None;
    }
}

// EXPERIMENT
fn baloons(wm: WindowManager) -> Result<(), PsychophysicsError> {
    // find all monitors available
    let monitors = wm.get_available_monitors();
    // get the second monitor if available, otherwise use the first one
    let monitor = monitors.get(1).unwrap_or(
        monitors
            .first()
            .expect("No monitor found - this should not happen"),
    );

    // choose the highest possible resolution for the given refresh rate
    let window_options: WindowOptions = WindowOptions::Windowed { resolution: None };

    // finally, create the window
    let window = wm.create_window(&window_options);

    // wait 1s to make sure the window is created (this should not be necessary but is a workaround for a bug in winit)
    std::thread::sleep(std::time::Duration::from_secs_f32(1.0));

    #[cfg(target_os = "ios")]
    let session = {
        // create a delegate for the ARSession that handles new frames
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new("MyARSessionDelegate", superclass).unwrap();

        extern "C" fn did_update_frame(obj: &Object, _: Sel, _: id, frame: id) {
            println!("did_update_frame");
            let pos = get_current_gaze_position(frame);
            // add the gaze position to the global variable
            *GAZE_POSITION.lock().unwrap() = pos;
        }

        unsafe {
            decl.add_method(
                sel!(session:didUpdateFrame:),
                did_update_frame as extern "C" fn(&Object, Sel, id, id),
            );
        }

        let delegate_class = decl.register();
        let delegate_object: id = unsafe { msg_send![delegate_class, new] };

        // run the session
        unsafe {
            let session: id = msg_send!(class!(ARSession), new);
            let _: () = msg_send![session, setDelegate: delegate_object];
            let configuration: id = msg_send![class!(ARFaceTrackingConfiguration), new];
            let _: () = msg_send![configuration, setWorldAlignment: 2usize];
            let _: () = msg_send![session, runWithConfiguration: configuration];
            StrongPtr::new(session)
        }
    };

    let image = Sprite::new_from_spritesheet(
        "/Users/marc/psychophysics/white-sails-rocking-action-25-frames-1317px-by-1437px-per-frame.png",
        5,
        5,
    )?;

    let rect = Circle::new(0.0, 0.0, 100.0);

    let mut image_stimulus = PatternStimulus::new(&window, rect, image);

    // create all the balloons
    let mut balloons = Vec::new();
    for i in 0..N_BALLOONS {
        // create a random float between 1 and 5
        let random_float = rand::random::<f64>() * 4.0 + 1.0;

        // create the pattern
        let gratings =
            GaborPatch::new(0.0, random_float, color::WHITE, (0.0, 0.0), (50.0, 50.0));

        // create a cirle
        let circle = Circle::new(0.0, 0.0, N_BALLOON_RADIUS as f64);

        // create the actual stimulus as a combination of the circle and the (masked) pattern
        let stimulus = PatternStimulus::new(&window, circle, gratings);

        // create a random velocity
        let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();
        let direction = Vector2::new(
            normal.sample(&mut rand::thread_rng()) as f32,
            normal.sample(&mut rand::thread_rng()) as f32,
        );

        // normalize the direction vector and scale it by the velocity
        let velocity = direction.normalize() * SPEED;

        // create the ballon
        let balloon = Balloon {
            position: Vector2::new(i as f32 * 310.0 + 305.0, 0.0), // this is the position of the balloon
            radius: N_BALLOON_RADIUS as f32, // this is the radius used for the collision detection
            velocity: velocity,              // this is the velocity of the balloon
            hidden: false,                   // this is used to hide the balloon
            stimulus,                        // this is the stimulus
        };

        // add the balloon to the vector of balloons
        balloons.push(balloon);
    }

    // create red circle for gaze position
    let circle = Circle::new(0.0, 0.0, 10.0);
    let red = ColorStimulus::new(&window, circle, color::RED);

    // get window size in pixels
    let box_size = Vector2::new(window.width_px() as f32, window.height_px() as f32);
    let box_origin = Vector2::new(-box_size.x / 2.0, -box_size.y / 2.0);

    // create the simulator
    let simulator = BalloonSimulator::new(balloons, box_origin, box_size).skip(10_000); // skip the first 10_000 steps to generate a pseudo-random state

    let mut i = 0;
    // run the simulation by stepping through the simulator
    for balloons_at_step in simulator {
        // obtain the frame and set the background color
        let mut frame = window.get_frame();
        frame.set_bg_color(color::GRAY);

        // iterate through the balloons and add the stimulus to the frame
        for balloon in balloons_at_step {
            if !balloon.hidden {
                // set new position for the stimulus
                balloon
                    .stimulus
                    .translate(balloon.position.x, balloon.position.y);

                // finally, add the stimulus to the frame
                frame.add(&balloon.stimulus);
            }
        }

        // compute the gaze position
        let gp = GAZE_POSITION.lock().unwrap();

        if gp.is_some() {
            let gaze_position = gp.unwrap() * 100.0;
            println!("new gaze position: {:?}", gaze_position);
            // set new position for the stimulus
            red.translate(gaze_position.y * 150.0, -gaze_position.x * 150.0);
        }

        // add the red circle for the gaze position
        frame.add(&red);
        frame.add(&image_stimulus);

        if i % 4 == 0 {
            image_stimulus.pattern.advance_image_index();
        }

        // submit the frame to the window for rendering
        window.submit_frame(frame);
        i = i + 1;
    }

    Ok(())
}

// this is the entry point for the mobile app
#[mobile_entry_point]
fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = pollster::block_on(ExperimentManager::new());

    em.run_experiment(baloons);
}

/// A balloon simulator
pub struct BalloonSimulator<T: Stimulus> {
    pub balloons: Vec<Balloon<T>>,
    pub collider_set: ColliderSet,
    pub balloon_set: RigidBodySet,
    pub balloon_handles: Vec<RigidBodyHandle>,
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub n_steps: usize,
}

/// A balloon
#[derive(Debug, Clone)]
pub struct Balloon<T: Stimulus> {
    pub position: Vector2<Real>,
    pub radius: Real,
    pub velocity: Vector2<Real>,
    pub hidden: bool,
    pub stimulus: T,
}

impl<T: Stimulus> BalloonSimulator<T> {
    /// Create a new balloon simulator
    pub fn new(
        balloons: Vec<Balloon<T>>,
        origin: Vector2<Real>,
        extend: Vector2<Real>,
    ) -> Self {
        // all the things that can collide with each other will live in the collider set
        let mut collider_set = ColliderSet::new();

        // create the walls (they usually align with the window size)
        let walls = ColliderBuilder::polyline(
            vec![
                Point::new(origin.x, origin.y),            // bottom left
                Point::new(origin.x, origin.y + extend.y), // top left
                Point::new(origin.x + extend.x, origin.y + extend.y), // top right
                Point::new(origin.x + extend.x, origin.y), // bottom right
                Point::new(origin.x, origin.y),            // bottom left
            ],
            None,
        )
        .restitution(1.0) // no energy loss
        .friction(0.0) // no friction
        .build();

        collider_set.insert(walls);

        // create the balloons as rigid bodies and add them to the collider set
        let mut balloon_set = RigidBodySet::new();
        let mut balloon_handles = Vec::new();

        // for each balloon, create a rigid body and a collider with the same size
        for balloon in balloons.iter() {
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(balloon.position)
                .linvel(balloon.velocity)
                .build();

            let collider = ColliderBuilder::ball(balloon.radius)
                .restitution(1.0)
                .friction(0.0)
                .build();

            let ball_body_handle = balloon_set.insert(rigid_body);
            balloon_handles.push(ball_body_handle);

            // insert the collider with the rigid body as parent
            collider_set.insert_with_parent(collider, ball_body_handle, &mut balloon_set);
        }

        // create the physics pipeline
        let gravity = vector![0.0, 0.0];
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / MONITOR_HZ as f32;
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();

        Self {
            balloons,
            collider_set,
            balloon_set,
            balloon_handles,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            n_steps: 1,
        }
    }
}

// implement the Iterator trait for the BalloonSimulator
// each iteration steps through n_steps of the simulation
impl<T: Stimulus + Clone> Iterator for BalloonSimulator<T> {
    type Item = Vec<Balloon<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..self.n_steps {
            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.balloon_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                Some(&mut self.query_pipeline),
                &(),
                &(),
            );
        }

        // update the balloons
        for (i, handle) in self.balloon_handles.iter().enumerate() {
            let balloon = &mut self.balloons[i];
            let rb = self.balloon_set.get(*handle).unwrap();
            balloon.position = rb.position().translation.vector;
            balloon.velocity = rb.linvel().clone();
        }
        Some(self.balloons.clone())
    }
}
