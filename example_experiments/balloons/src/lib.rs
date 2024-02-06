use mobile_entry_point::mobile_entry_point;
use nalgebra::Vector2;
use psychophysics::{
    prelude::*,
    visual::{
        color::RawRgba,
        stimuli::{patterns::SineGratings, Stimulus},
    },
    ExperimentManager, WindowManager, WindowOptions,
};
use rand_distr::Distribution;
use rapier2d::prelude::*;

const N_BALLOONS: usize = 5;
const N_BALLOON_RADIUS: f32 = 150.0;
const SPEED: f32 = 100.0;

const MONITOR_HZ: f64 = 60.0;

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
    let window_options: WindowOptions =
        WindowOptions::FullscreenHighestResolution {
            monitor: Some(monitor.clone()),
            refresh_rate: Some(MONITOR_HZ),
        };

    // finally, create the window
    let window = wm.create_window(&window_options);

    // wait 1s to make sure the window is created (this should not be necessary but is a workaround for a bug in winit)
    std::thread::sleep(std::time::Duration::from_secs_f32(1.0));

    // create all the balloons
    let mut balloons = Vec::new();
    for i in 0..N_BALLOONS {
        // create a random float between 1 and 5
        let random_float = rand::random::<f64>() * 4.0 + 1.0;
        // create the pattern for the PatternStimulus
        let grating = SineGratings::new(
            0.0,
            Size::Pixels(random_float),
            RawRgba::new(1.0, 1.0, 1.0, 0.5),
        );

        // add a 2D Gaussian alpha mask to the pattern
        // this is just a normalised 2D Gaussian function pixel-wise multiplied with the alpha value
        let masked_grating =
            GaussianAlphamask::new(grating, (0.0, 0.0), (40.0, 40.0));

        // create the actual stimulus
        let stimulus = PatternStimulus::new(
            &window, // the window we want to display the stimulus inSetting color to
            psychophysics::visual::geometry::Circle::new(
                0.0,
                0.0,
                N_BALLOON_RADIUS as f64,
            ),
            masked_grating,
        );

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
            velocity: velocity, // this is the velocity of the balloon
            hidden: false,      // this is used to hide the balloon
            stimulus,           // this is the stimulus
        };

        // add the balloon to the vector of balloons
        balloons.push(balloon);
    }

    // get window size in pixels
    let window_size =
        Vector2::new(window.width_px() as f32, window.height_px() as f32);
    let origin = Vector2::new(-window_size.x / 2.0, -window_size.y / 2.0);

    // create the simulator
    let simulator =
        BalloonSimulator::new(balloons, origin, window_size).skip(10_000); // skip the first 10_000 steps to generate a pseudo-random state

    // run the simulation by stepping through the simulator
    for balloons in simulator {
        // obtain the frame and set the background color
        let mut frame = window.get_frame();
        frame.set_bg_color(color::GRAY);

        // iterate through the balloons and add the stimulus to the frame
        for balloon in balloons.iter() {
            if !balloon.hidden {
                // create a transformation to move the stimulus to the correct position
                let transform = Transformation2D::Translation(
                    Size::Pixels(balloon.position.x as f64),
                    Size::Pixels(balloon.position.y as f64),
                );
                // set the transformation
                balloon.stimulus.set_transformation(transform);
                // finally, add the stimulus to the frame
                frame.add(&balloon.stimulus);
            }
        }
        // submit the frame to the window for rendering
        window.submit_frame(frame);
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
                Point::new(origin.x, origin.y), // bottom left
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
            collider_set.insert_with_parent(
                collider,
                ball_body_handle,
                &mut balloon_set,
            );
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
