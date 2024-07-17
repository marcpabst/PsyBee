use uuid::Uuid;

use dyn_clone::DynClone;

use super::{
    geometry::{Size, Transformation2D},
    window::Window,
};

use pyo3::prelude::*;

use crate::renderer::Renderable;

pub mod gabor;
pub mod image;
pub mod sprite;

/// The stimulus trait.
pub trait Stimulus: Send + Sync + downcast_rs::Downcast + std::fmt::Debug + dyn_clone::DynClone {
    /// Draw the stimulus and produce a list of drawables.
    fn draw(&self, window: &Window) -> Vec<Renderable>;

    /// Check if the stimulus contains a specific Point.
    fn contains(&self, x: Size, y: Size) -> bool {
        // by default, stimuli will report false for contains
        false
    }

    /// Return the UUID that identifies the stimulus.
    fn uuid(&self) -> Uuid;

    /// Check if two stimuli are equal.
    fn equal(&self, other: &dyn Stimulus) -> bool {
        self.uuid() == other.uuid()
    }

    /// Set the origin of the stimulus.
    fn set_origin(&mut self, x: Size, y: Size) -> ();

    /// Get the current origin of the stimulus.
    fn origin(&self) -> (Size, Size);

    /// Returns true if the stimulus is currently visible.
    fn visible(&self) -> bool {
        true
    }

    /// Set the visibility of the stimulus.
    fn set_visible(&mut self, visible: bool) {
        // do nothing by default
    }

    /// Hide the stimulus. This is a convenience method that calls
    /// `set_visible(false)`.
    fn hide(&mut self) -> () {
        self.set_visible(false);
    }

    /// Show the stimulus. This is a convenience method that calls
    /// `set_visible(true)`.
    fn show(&mut self) -> () {
        self.set_visible(true);
    }

    /// Toggle the visibility of the stimulus.
    fn toggle_visibility(&mut self) -> () {
        self.set_visible(!self.visible());
    }

    /// Set the transformation.
    fn set_transformation(&mut self, transformation: Transformation2D) -> ();

    /// Add a transformation to the current transformation.
    fn add_transformation(&mut self, transformation: Transformation2D) -> ();

    /// Translate the object by the given x and y coordinates.
    fn translate(&mut self, x: Size, y: Size) {
        self.add_transformation(Transformation2D::Translation(x, y));
    }

    /// Set the translation of the object to the given x and y coordinates. This
    /// overwrites any previously applied transformations.
    fn set_translation(&mut self, x: Size, y: Size) {
        self.set_transformation(Transformation2D::Translation(x, y));
    }

    /// Rotate the object around the center of the object by the given angle.
    fn rotate_center(&mut self, angle: f32) {
        self.set_transformation(Transformation2D::RotationCenter(angle));
    }

    /// Rotate the object around the given point by the given angle.
    fn rotate_point(&mut self, angle: f32, x: Size, y: Size) {
        self.set_transformation(Transformation2D::RotationPoint(angle, x, y));
    }

    /// Scale the object around the center of the object by the given x and y
    /// factors.
    fn scale_center(&mut self, x: f32, y: f32) {
        self.set_transformation(Transformation2D::ScaleCenter(x, y));
    }

    /// Scale the object around the given point by the given x and y factors.
    fn scale_point(&mut self, x: f32, y: f32, x0: Size, y0: Size) {
        self.set_transformation(Transformation2D::ScalePoint(x, y, x0, y0));
    }

    /// Shear the object around the center of the object by the given x and y
    /// factors.
    fn shear_center(&mut self, x: f32, y: f32) {
        self.set_transformation(Transformation2D::ShearCenter(x, y));
    }

    /// Shear the object around the given point by the given x and y factors.
    fn shear_point(&mut self, x: f32, y: f32, x0: Size, y0: Size) {
        self.set_transformation(Transformation2D::ShearPoint(x, y, x0, y0));
    }
}

downcast_rs::impl_downcast!(Stimulus);

// implement dyn_clone::DynClone for Stimulus
dyn_clone::clone_trait_object!(Stimulus);

/// Wraps a Stimulus. This class is used either as a base class for other
/// stimulus classes or as a standalone class, when no specific runtume type
/// information is available.
#[pyclass(name = "Stimulus", subclass)]
#[derive(Debug, Clone)]
pub struct PyStimulus(pub Box<dyn Stimulus + 'static>);

/// A group of stimuli.
#[derive(Debug, Clone)]
pub struct StimulusGroup {
    /// The UUID of the stimulus group.
    id: Uuid,
    visible: bool,
    origin: (Size, Size),

    /// The stimuli in the group.
    pub stimuli: Vec<Box<dyn Stimulus>>,
}

#[pymethods]
impl PyStimulus {
    fn set_transformation(&mut self, transformation: Transformation2D) {
        self.0.set_transformation(transformation);
    }
}

impl StimulusGroup {
    /// Create a new stimulus group.
    pub fn new(origin: (Size, Size)) -> StimulusGroup {
        StimulusGroup {
            id: Uuid::new_v4(),
            visible: true,
            origin,
            stimuli: vec![],
        }
    }

    /// Add a stimulus to the group.
    pub fn add(&mut self, stimulus: Box<dyn Stimulus>) {
        self.stimuli.push(stimulus);
    }

    /// Remove a stimulus from the group.
    pub fn remove(&mut self, stimulus: &dyn Stimulus) {
        self.stimuli.retain(|s| !s.equal(stimulus));
    }
}

impl Stimulus for StimulusGroup {
    fn draw(&self, window: &Window) -> Vec<Renderable> {
        let mut drawables = vec![];
        for stimulus in &self.stimuli {
            drawables.extend(stimulus.draw(window));
        }
        drawables
    }

    fn uuid(&self) -> Uuid {
        self.id
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_origin(&mut self, x: Size, y: Size) -> () {
        self.origin = (x, y);
    }

    fn origin(&self) -> (Size, Size) {
        self.origin.clone()
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        todo!()
    }

    fn add_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        todo!()
    }
}

/// The alignment of a grid, i.e., where the grid is anchored relative to the
/// origin.
#[derive(Debug, Clone, Copy)]
pub enum GridAlignment {
    CenterCenter,
    CenterTop,
    CenterBottom,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// A grid of stimuli.
#[derive(Debug)]
pub struct StimulusGrid {
    /// The UUID of the stimulus group.
    id: Uuid,
    visible: bool,
    origin: (Size, Size),

    /// The alignment of the grid.
    pub alignment: GridAlignment,
    /// Number of rows in the grid.
    pub rows: Option<usize>,
    /// Number of columns in the grid.
    pub columns: Option<usize>,
    /// The spacing between stimuli in the grid.
    pub spacing: (Size, Size),

    /// The stimuli in the grid.
    pub stimuli: Vec<Vec<Box<dyn Stimulus>>>,
}

/// A Value. Values wrap a type and provide a way to share the same value
/// between multiple stimuli properties. This is extremely useful for creating
/// groups of stimuli that share the same properties. A value can be cloned
/// and passed to multiple stimuli, while atomic reference counting ensures
/// that the value is only dropped when the last reference is dropped (RefCell).
#[derive(Clone, Debug)]
pub struct Value<T: Clone> {
    inner: std::sync::Arc<std::cell::RefCell<T>>,
}

impl<T: Clone> Value<T> {
    /// Create a new value.
    pub fn new(value: T) -> Self {
        Value {
            inner: std::sync::Arc::new(std::cell::RefCell::new(value)),
        }
    }

    /// Get the value.
    pub fn get(&self) -> T {
        self.inner.borrow().clone()
    }

    /// Set the value.
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
    }
}
