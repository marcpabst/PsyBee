use mobile_entry_point::mobile_entry_point;
use nalgebra::Vector2;
use psychophysics::{
    prelude::*,
    visual::{
        color::RawRgba,
        stimuli::{patterns::SineGratings, PatternStimulus},
    },
    ExperimentManager, WindowManager, WindowOptions,
};
use rand_distr::Distribution;
use rapier2d::prelude::*;

const N_BALLOONS: usize = 10;
const N_BALLOON_RADIUS: f32 = 150.0;

const MONITOR_HZ: f64 = 60.0;

// EXPERIMENT
fn baloons(wm: WindowManager) -> Result<(), PsychophysicsError> {
    let monitors = wm.get_available_monitors();
    let monitor = monitors
        .get(1)
        .unwrap_or(monitors.first().expect("No monitor found!"));

    let window_options: WindowOptions = WindowOptions::FullscreenHighestResolution {
        monitor: Some(monitor.clone()),
        refresh_rate: Some(MONITOR_HZ),
    };

    let window = wm.create_window(&window_options);

    // wait 1s to make sure the window is created
    std::thread::sleep(std::time::Duration::from_secs(1));
    log::info!("Window created");
    // create balloons
    let mut balloons = Vec::new();
    for i in 0..N_BALLOONS {
        // create new circle grating stimulus
        let stimulus = PatternStimulus::new(
            &window, // the window we want to display the stimulus inSetting color to
            psychophysics::visual::geometry::Circle::new(
                -Size::ScreenWidth(0.5),
                Size::ScreenHeight(0.5),
                N_BALLOON_RADIUS as f64,
            ),
            SineGratings::new(0.0, Size::Pixels(5.0), RawRgba::new(1.0, 1.0, 1.0, 0.01)),
        );

        // crrate random initial direction
        let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();
        let direction = Vector2::new(
            normal.sample(&mut rand::thread_rng()) as f32,
            normal.sample(&mut rand::thread_rng()) as f32,
        );
        // normalize the direction vector and scale it by the velocity
        let velocity = direction.normalize() * 100.0;

        balloons.push(Balloon {
            position: Vector2::new(i as f32 * (N_BALLOON_RADIUS * 2.0) + 400.0, 1000.0),
            radius: N_BALLOON_RADIUS as f32,
            velocity: velocity,
            hidden: false,
            stimulus,
        });
    }

    // get window size in pixels using window.get_height_px() and window.get_width_px()
    let window_size =
        Vector2::new(window.get_width_px() as f32, window.get_height_px() as f32);

    // create the simulator
    let simulator =
        BalloonSimulator::new(balloons, Vector2::new(0.0, 0.0), window_size).skip(10_000); // skip the first 10_000 steps to generate a pseudo-random state

    // run the simulation
    for balloons in simulator {
        let mut frame = window.get_frame();
        frame.set_bg_color(color::GRAY);
        for (i, balloon) in balloons.iter().enumerate() {
            if !balloon.hidden {
                let stim = balloon.stimulus.clone();
                // update the stimulus position by setting the transformation
                let transform = Transformation2D::Translation(
                    Size::Pixels(balloon.position.x as f64),
                    -Size::Pixels(balloon.position.y as f64),
                );
                stim.set_transformation(transform);
                frame.add(&stim);
            }
        }

        window.submit_frame(frame);
    }

    Ok(())
}

// ENTRY POINT
#[mobile_entry_point]
fn main() {
    // start experiment (this will block until the experiment is finished)
    let mut em = pollster::block_on(ExperimentManager::new());

    em.run_experiment(baloons);
}

pub struct BalloonSimulator {
    pub balloons: Vec<Balloon>,
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

#[derive(Debug, Clone)]
pub struct Balloon {
    pub position: Vector2<Real>,
    pub radius: Real,
    pub velocity: Vector2<Real>,
    pub hidden: bool,
    pub stimulus: PatternStimulus<SineGratings>,
}

impl BalloonSimulator {
    pub fn new(
        balloons: Vec<Balloon>,
        bbox_origin: Vector2<Real>,
        bbox_size: Vector2<Real>,
    ) -> Self {
        // set of balloons
        let mut collider_set = ColliderSet::new();

        let walls = ColliderBuilder::polyline(
            vec![
                Point::new(bbox_origin.x, bbox_origin.y), // bottom left
                Point::new(bbox_origin.x, bbox_size.y),   // top left
                Point::new(bbox_size.x, bbox_size.y),     // bottom right
                Point::new(bbox_size.x, bbox_origin.y),   // top right
                Point::new(bbox_origin.x, bbox_origin.y), // bottom left
            ],
            None,
        )
        .restitution(1.0)
        .friction(0.0)
        .build();
        collider_set.insert(walls);

        /* Create the bouncing balls */
        let mut balloon_set = RigidBodySet::new();
        let mut balloon_handles = Vec::new();

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
            collider_set.insert_with_parent(collider, ball_body_handle, &mut balloon_set);
        }

        /* Create other structures necessary for the simulation. */
        let gravity = vector![0.0, 0.0];
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / 60.0;
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
impl Iterator for BalloonSimulator {
    type Item = Vec<Balloon>;

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
