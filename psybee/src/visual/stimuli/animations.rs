use std::time::Instant;

use pyo3::{types::PyAnyMethods, Bound, FromPyObject, PyAny, PyResult};

use super::{Stimulus, StimulusParamValue};
use crate::visual::{
    geometry::Size,
    window::{Window, WindowState},
};

#[derive(FromPyObject, Debug, Clone)]
pub enum Repeat {
    /// Play the animation the specified number of times.
    Loop(u32),
    /// Ping-pong the animation the specified number of times.
    PingPong(u32),
}

#[derive(Debug, Clone)]
pub enum TransitionFunction {
    /// No transition function.
    None,
    /// A linear transition function.
    Linear(f64, f64),
    /// A cubic bezier transition function.
    CubicBezier(f64, f64, f64, f64),
}

// implement FromPyObject for TransitionFunction
impl<'py> FromPyObject<'py> for TransitionFunction {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // try to extract a string from the object and then convert it to a TransitionFunction
        if let Ok(name) = ob.extract::<String>() {
            Ok(TransitionFunction::from_str(&name))
        } else {
            // if the object is not a string, try to extract a tuple of f64s
            let tuple = ob.extract::<(f64, f64, f64, f64)>()?;
            Ok(TransitionFunction::CubicBezier(tuple.0, tuple.1, tuple.2, tuple.3))
        }
    }
}

impl TransitionFunction {
    pub fn linear() -> Self {
        Self::Linear(0.0, 1.0)
    }

    pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self::CubicBezier(x1, y1, x2, y2)
    }

    pub fn ease_in() -> Self {
        Self::CubicBezier(0.42, 0.0, 1.0, 1.0)
    }

    pub fn ease_out() -> Self {
        Self::CubicBezier(0.0, 0.0, 0.58, 1.0)
    }

    pub fn ease_in_out() -> Self {
        Self::CubicBezier(0.42, 0.0, 0.58, 1.0)
    }

    pub fn from_str(name: &str) -> Self {
        match name {
            "linear" => Self::linear(),
            "ease-in" => Self::ease_in(),
            "ease-out" => Self::ease_out(),
            "ease-in-out" => Self::ease_in_out(),
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    /// The name of the attribute that should be animated.
    paramter: String,
    /// The value that the attribute should be animated from.
    from: StimulusParamValue,
    /// The value that the attribute should be animated to.
    to: StimulusParamValue,
    /// The duration of the animation in seconds.
    duration: f64,
    /// The time at which the animation should start from when it is created.
    start_time: Instant,
    /// Repeat the animation according to the specified repeat mode.
    repeat: Repeat,
    /// The easing function that should be used for the animation.
    easing: TransitionFunction,
}

impl Animation {
    pub fn new(
        parameter: &str,
        from: StimulusParamValue,
        to: StimulusParamValue,
        duration: f64,
        start_time: Instant,
        repeat: Repeat,
        easing: TransitionFunction,
    ) -> Self {
        Self {
            paramter: parameter.to_string(),
            from,
            to,
            duration,
            start_time,
            repeat,
            easing,
        }
    }

    /// Returns the name of the attribute that should be animated.
    pub fn parameter(&self) -> &str {
        &self.paramter
    }

    /// Returns the current value of the animated parameter at the specified time (f64).
    pub fn value_f64(from: f64, to: f64, elapsed: f64, duration: f64, easing: TransitionFunction) -> f64 {
        let t = elapsed / duration;
        let t = match easing {
            TransitionFunction::None => t,
            TransitionFunction::Linear(a, b) => a + (b - a) * t,
            TransitionFunction::CubicBezier(p1, p2, p3, p4) => {
                let t2 = t * t;
                let t3 = t2 * t;
                let c = 3.0 * (p1 - p2);
                let b = 3.0 * (p3 - p1) - c;
                let a = 1.0 - c - b;
                a * t3 + b * t2 + c * t
            }
        };

        from + (to - from) * t
    }

    /// Returns the current value of the animated parameter at the specified time.
    pub fn value(&self, time: Instant, window_state: &WindowState) -> StimulusParamValue {
        if self.finished(time) {
            return self.to.clone();
        }

        // let elapsed = time.duration_since(self.start_time).as_secs_f64();
        let elapsed = match self.repeat {
            Repeat::Loop(n) => {
                let elapsed = time.duration_since(self.start_time).as_secs_f64();
                elapsed % self.duration
            }
            Repeat::PingPong(n) => {
                let elapsed = time.duration_since(self.start_time).as_secs_f64();
                let elapsed = elapsed % (self.duration * 2.0);
                if elapsed > self.duration {
                    self.duration - (elapsed - self.duration)
                } else {
                    elapsed
                }
            }
        };

        let duration = self.duration;
        let easing = self.easing.clone();
        let from = self.from.clone();
        let to = self.to.clone();

        let window_size = window_state.size;
        let screen_props = window_state.physical_screen;

        match (from, to) {
            (StimulusParamValue::f64(f), StimulusParamValue::f64(t)) => {
                StimulusParamValue::f64(Self::value_f64(f, t, elapsed, duration, easing))
            }
            (StimulusParamValue::Size(f), StimulusParamValue::Size(t)) => {
                let f = f.eval(window_size, screen_props) as f64;
                let t = t.eval(window_size, screen_props) as f64;
                let value = Self::value_f64(f, t, elapsed, duration, easing);
                StimulusParamValue::Size(Size::Pixels(value as f32))
            }
            _ => self.to.clone(),
        }
    }

    /// Returns whether the animation has finished.
    pub fn finished(&self, time: Instant) -> bool {
        match self.repeat {
            Repeat::Loop(n) => {
                let elapsed = time.duration_since(self.start_time).as_secs_f64();
                elapsed > self.duration * n as f64
            }
            Repeat::PingPong(n) => {
                let elapsed = time.duration_since(self.start_time).as_secs_f64();
                elapsed > self.duration * n as f64 * 2.0
            }
        }
    }
}
