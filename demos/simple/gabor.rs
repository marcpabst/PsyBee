use super::{impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParam, StimulusParams};
use crate::{
    prelude::{Size, Transformation2D},
    visual::window::Window,
};
use psybee_proc::StimulusParams;
use pyo3::{exceptions::PyValueError, prelude::*};
use renderer::prelude::*;
use uuid::Uuid;

#[derive(StimulusParams, Clone, Debug)]
pub struct GaborParams {
    pub cx: Size,
    pub cy: Size,
    pub radius: Size,
    pub cycle_length: Size,
    pub phase: f64,
    pub sigma: Size,
    pub orientation: f64,
}

#[derive(Clone, Debug)]
pub struct GaborStimulus {
    id: uuid::Uuid,

    params: GaborParams,

    grating_sine_colors: Vec<RGBA>,
    gaussian_colors: Vec<RGBA>,

    transformation: Transformation2D,
    visible: bool,
}

impl GaborStimulus {
    pub fn new(
        cx: Size,
        cy: Size,
        radius: Size,
        cycle_length: Size,
        phase: f64,
        sigma: Size,
        orientation: f64,
    ) -> Self {
        let sine_grating_colors: Vec<RGBA> = (0..256)
            .map(|i| {
                let x = i as f32 / 256.0 * 1.0 * std::f32::consts::PI;
                let t = x.sin();
                RGBA {
                    r: t,
                    g: t,
                    b: t,
                    a: 1.0,
                }
            })
            .collect();

        let gaussian_colors: Vec<RGBA> = (0..256)
            .map(|i| {
                let sigma: f32 = 0.25;
                // we need a Gaussian function scaled to values between 0 and 1
                // i.e., f(x) = exp(-x^2 / (2 * sigma^2))
                let x = (i as f32 / 256.0);
                let t = (-x.powi(2) / (2.0 * sigma.powi(2))).exp();
                RGBA {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: t,
                }
            })
            .collect();
        Self {
            id: Uuid::new_v4(),
            transformation: crate::visual::geometry::Transformation2D::Identity(),
            visible: true,

            params: GaborParams {
                cx,
                cy,
                radius,
                cycle_length,
                phase,
                sigma,
                orientation,
            },
            grating_sine_colors: sine_grating_colors,
            gaussian_colors,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "GaborStimulus", extends=PyStimulus)]
pub struct PyGaborStimulus();

#[pymethods]
impl PyGaborStimulus {
    #[new]
    #[pyo3(signature = (
        cx,
        cy,
        radius,
        cycle_lenght,
        sigma,
        phase = 0.0,
        orientation = 0.0
    ))]

    fn __new__(
        cx: Size,
        cy: Size,
        radius: Size,
        cycle_lenght: Size,
        sigma: Size,
        phase: f64,
        orientation: f64,
    ) -> (Self, PyStimulus) {
        (
            Self(),
            PyStimulus(Box::new(GaborStimulus::new(
                cx,
                cy,
                radius,
                cycle_lenght,
                phase,
                sigma,
                orientation,
            ))),
        )
    }
}

impl_pystimulus_for_wrapper!(PyGaborStimulus, GaborStimulus);

impl Stimulus for GaborStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&self, scene: &mut VelloScene, window: &Window) {
        if !self.visible {
            return;
        }

        // convert physical units to pixels
        let radius = self.params.radius.eval(&window.physical_properties);
        let sigma = self.params.sigma.eval(&window.physical_properties);
        let cycle_length = self.params.cycle_length.eval(&window.physical_properties) as f64;
        let pos_x = self.params.cx.eval(&window.physical_properties) as f64;
        let pos_y = self.params.cy.eval(&window.physical_properties) as f64;

        let trans_mat = self
            .transformation
            .to_transformation_matrix(&window.physical_properties);

        // convert phase into the range [0, 1] (from [0, 2π])
        let phase = self.params.phase % (2.0 * std::f64::consts::PI);
        let transl_x = phase * cycle_length;
        let transl_x = 0.0;

        // transform for the brush
        let grating_transform = Affine::rotate_at(self.params.orientation, pos_x + transl_x, pos_y);

        let sine_grating = Geom {
            style: Style::Fill(FillStyle::NonZero),
            shape: Circle {
                center: Point { x: pos_x, y: pos_y },
                radius: radius as f64,
            },
            brush: Brush::Gradient(Gradient::new_equidistant(
                Extend::Repeat,
                GradientKind::Linear {
                    start: Point {
                        x: pos_x + transl_x,
                        y: pos_y,
                    },
                    end: Point {
                        x: pos_x + cycle_length + transl_x,
                        y: pos_y,
                    },
                },
                &self.grating_sine_colors,
            )),
            transform: Affine::identity(),
            brush_transform: Some(grating_transform),
        };

        let gaussian = Geom {
            style: Style::Fill(FillStyle::NonZero),
            shape: Circle {
                center: Point { x: pos_x, y: pos_y },
                radius: radius as f64,
            },
            brush: Brush::Gradient(Gradient::new_equidistant(
                Extend::Pad,
                GradientKind::Radial {
                    start_center: Point { x: pos_x, y: pos_y },
                    start_radius: 0.0,
                    end_center: Point { x: pos_x, y: pos_y },
                    end_radius: sigma,
                },
                &self.gaussian_colors,
            )),
            transform: Affine::identity(),
            brush_transform: None,
        };

        scene.draw_alpha_mask(
            |scene| {
                scene.draw(sine_grating);
            },
            |scene| {
                scene.draw(gaussian);
            },
            Circle {
                center: Point { x: pos_x, y: pos_y },
                radius: radius as f64,
            },
            Affine::identity(),
        );
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation;
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transformation = transformation * self.transformation.clone();
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transformation.clone()
    }

    fn contains(&self, x: Size, y: Size, window: &Window) -> bool {
        // let props = &window.physical_properties;
        // // eval to pixels
        // let x = x.eval(props);
        // let y = y.eval(props);
        // let size = self.size.eval(props);

        // // apply the transformation matrix
        // let (xn, yn) = self.transform_point(x, y, window);

        // // check if the point is within the stimulus
        // xn >= -size && xn <= size && yn >= -size && yn <= size
        false
    }

    fn get_param(&self, name: &str) -> Option<StimulusParam> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParam) {
        self.params.set_param(name, value)
    }
}
