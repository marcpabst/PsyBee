use std::{ops::Deref, str::FromStr, sync::Arc};

use pyo3::{pyclass, pymethods, types::PyAnyMethods, FromPyObject};
use strum::{EnumString, VariantArray, VariantNames};
use web_time::SystemTime;
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::{event as winit_event, keyboard::Key};

use crate::visual::{geometry::Size, window::Window};

// pub mod video;

/// A mouse button.
#[derive(Debug, Clone, PartialEq)]
#[pyclass]
pub enum MouseButton {
    /// The left mouse button.
    Left(),
    /// The right mouse button.
    Right(),
    /// The middle mouse button.
    Middle(),
    /// The forward mouse button.
    Forward(),
    /// The back mouse button.
    Back(),
    /// An additional mouse button with the given index.
    Other(u16),
}

#[derive(Debug, Clone, enum_fields::EnumFields, strum::EnumDiscriminants)]
#[pyclass(unsendable)]
#[strum_discriminants(
    name(EventKind),
    strum(serialize_all = "snake_case"),
    derive(EnumString, VariantNames)
)]
pub enum Event {
    /// A keypress event. This is triggered when a key is pressed.
    KeyPress {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// String representation of the key that was pressed.
        key: String,
        /// KeyCode of the key that was pressed.
        code: u32,
    },
    /// A key release event. This is triggered when a key is released.
    KeyRelease {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// String representation of the key that was released.
        key: String,
        /// KeyCode of the key that was released.
        code: u32,
    },

    /// A mouse button press event. This is triggered when a mouse button is
    /// pressed.
    MouseButtonPress {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The button that was pressed.
        button: MouseButton,
        /// The position of the mouse cursor when the button was pressed.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
    },

    /// A mouse button release event. This is triggered when a mouse button is
    /// released.
    MouseButtonRelease {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The button that was released.
        button: MouseButton,
        /// The position of the mouse cursor when the button was released.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// A touch start event. This is triggered when a touch screen is touched.
    TouchStart {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The position of the touch.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// A touch move event. This is triggered when a touch screen is moved.
    TouchMove {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The position of the touch.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// A touch end event. This is triggered when a touch screen is
    /// released.
    TouchEnd {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The position of the touch.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// A touch cancel event. This is triggered when a touch screen is
    /// cancelled.
    TouchCancel {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The position of the touch.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// The window has lost focus.
    FocusGained {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// The window has gained focus.
    FocusLost {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// The mouse cursor was moved.
    CursorMoved {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The position of the cursor.
        position: (f32, f32),
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// The mouse cursor was entered into the window.
    CursorEntered {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// The mouse cursor was exited from the window.
    CursorExited {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// A pressure-sensitive touchpad was pressed (if available).
    TouchpadPress {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The pressure of the touch.
        pressure: f32,
        /// The level of the touch.
        stage: i64,
        /// The Window that the event was triggered on.
        window: Window,
    },
    /// The mouse wheel was scrolled (or the equivalent touchpad gesture).
    MouseWheel {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The amount of horizontal scrolling.
        horizontal: f32,
        /// The amount of vertical scrolling.
        vertical: f32,
    },
    /// Any other event. The string contains the name of the event.
    Other {
        /// Timestamp of the event.
        timestamp: SystemTime,
        /// The name of the event.
        name: String,
    },
}

// Implements convenience methods for Event.
impl Event {
    /// Returns true if this element represents a press of the given key.
    pub fn key_pressed(&self, key: &str) -> bool {
        matches!(&self, Self::KeyPress { key: k, .. } if k == key)
    }

    /// Returns true if this element represents a release of the given key.
    pub fn key_released(&self, key: &str) -> bool {
        matches!(&self, Self::KeyRelease { key: k, .. } if k == key)
    }

    /// Returns true if this element represents a press of the given mouse
    /// button.
    pub fn mouse_button_pressed(&self, button_a: MouseButton) -> bool {
        matches!(&self, Self::MouseButtonPress { button, .. } if button_a == *button)
    }

    /// Returns true if this element represents a release of the given mouse
    /// button.
    pub fn mouse_button_released(&self, button_a: MouseButton) -> bool {
        matches!(&self, Self::MouseButtonRelease { button, .. } if button_a == *button)
    }

    /// Returns the kind of this event.
    pub fn kind(&self) -> EventKind {
        self.into()
    }
}

#[pymethods]
impl Event {
    #[getter]
    #[pyo3(name = "position")]
    fn py_position(&self) -> Option<(f32, f32)> {
        self.position().cloned()
    }

    #[getter]
    #[pyo3(name = "window")]
    fn py_window(&self) -> Option<Window> {
        self.window().cloned()
    }

    #[getter]
    #[pyo3(name = "key")]
    fn py_key(&self) -> Option<String> {
        self.key().cloned()
    }

    #[getter]
    #[pyo3(name = "id")]
    fn py_id(&self) -> Option<u64> {
        self.id().cloned().flatten()
    }
}

// Custom conversion from winit events to InputEvents.
pub(crate) trait EventTryFrom<T>: Sized {
    type Error;

    fn try_from_winit(value: T, window: &Window) -> Result<Self, Self::Error>;
}

/// Convert a winit WindowEvent to an InputEvent.
impl EventTryFrom<winit_event::WindowEvent> for Event {
    type Error = &'static str;

    fn try_from_winit(event: winit_event::WindowEvent, window: &Window) -> Result<Self, Self::Error> {
        let timestamp = SystemTime::now();
        let data = match event {
            // match keyboad events
            winit_event::WindowEvent::KeyboardInput {
                device_id: _, event, ..
            } => {
                let key = event.logical_key;

                let key_str = match key {
                    Key::Named(key) => Some(format!("{:?}", key).to_string()),
                    Key::Character(key) => Some(key.to_string()),
                    _ => None,
                };

                let key_str = match key_str {
                    Some(key) => key,
                    None => return Err("Failed to convert key to string"),
                };

                let key_code = u32::default();

                #[cfg(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "dragonfly",
                    target_os = "openbsd",
                    target_os = "netbsd"
                ))]
                let key_code = event.physical_key.to_scancode().unwrap_or_default();

                match event.state {
                    winit_event::ElementState::Pressed => Event::KeyPress {
                        timestamp,
                        key: key_str.to_string(),
                        code: key_code,
                    },
                    winit_event::ElementState::Released => Event::KeyRelease {
                        timestamp,
                        key: key_str.to_string(),
                        code: key_code,
                    },
                }
            }
            // match mouse button events
            winit_event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let button = match button {
                    winit_event::MouseButton::Left => MouseButton::Left(),
                    winit_event::MouseButton::Right => MouseButton::Right(),
                    winit_event::MouseButton::Middle => MouseButton::Middle(),
                    winit_event::MouseButton::Forward => MouseButton::Forward(),
                    winit_event::MouseButton::Back => MouseButton::Back(),
                    winit_event::MouseButton::Other(id) => MouseButton::Other(id),
                };

                let position = window.mouse_position().unwrap_or((0.0, 0.0));

                match state {
                    winit_event::ElementState::Pressed => Event::MouseButtonPress {
                        timestamp,
                        button,
                        position,
                        window: window.clone(),
                    },
                    winit_event::ElementState::Released => Event::MouseButtonRelease {
                        timestamp,
                        button,
                        position,
                        window: window.clone(),
                    },
                }
            }
            // match touch events
            winit_event::WindowEvent::Touch(touch) => {
                //  let position = (Size::Pixels(position.x) - Size::ScreenWidth(0.5), Size::Pixels(-position.y) + Size::ScreenHeight(0.5));
                let position = (touch.location.x as f32, touch.location.y as f32);

                // move by x_origin and y_origin
                let window_state = window.state.lock().unwrap();
                let window_size = window_state.size;
                let position = (
                    position.0 - (window_size.width as f32 / 2.0),
                    position.1 - (window_size.height as f32 / 2.0),
                );

                // dispatch on TouchPhase
                match touch.phase {
                    winit_event::TouchPhase::Started => Event::TouchStart {
                        timestamp,
                        position,
                        window: window.clone(),
                        id: Some(touch.id),
                    },
                    winit_event::TouchPhase::Moved => Event::TouchMove {
                        timestamp,
                        position,
                        window: window.clone(),
                        id: Some(touch.id),
                    },
                    winit_event::TouchPhase::Ended => Event::TouchEnd {
                        timestamp,
                        position,
                        window: window.clone(),
                        id: Some(touch.id),
                    },
                    winit_event::TouchPhase::Cancelled => Event::TouchCancel {
                        timestamp,
                        position,
                        window: window.clone(),
                        id: Some(touch.id),
                    },
                }
            }
            // match focus events
            winit_event::WindowEvent::Focused(focused) => {
                if focused {
                    Event::FocusGained {
                        timestamp,
                        window: window.clone(),
                    }
                } else {
                    Event::FocusLost {
                        timestamp,
                        window: window.clone(),
                    }
                }
            }
            // match cursor movement events
            winit_event::WindowEvent::CursorMoved { position, .. } => {
                let position = (position.x as f32, position.y as f32);

                // move by x_origin and y_origin
                let window_state = window.state.lock().unwrap();
                let window_size = window_state.size;
                let position = (
                    position.0 - (window_size.width as f32 / 2.0),
                    position.1 - (window_size.height as f32 / 2.0),
                );

                Event::CursorMoved {
                    timestamp,
                    position,
                    window: window.clone(),
                }
            }
            // match cursor enter events
            winit_event::WindowEvent::CursorEntered { .. } => Event::CursorEntered {
                timestamp,
                window: window.clone(),
            },
            // match cursor exit events
            winit_event::WindowEvent::CursorLeft { .. } => Event::CursorExited {
                timestamp,
                window: window.clone(),
            },
            // match touchpad press events
            winit_event::WindowEvent::TouchpadPressure {
                device_id: _,
                pressure,
                stage,
            } => Event::TouchpadPress {
                timestamp,
                pressure,
                stage,
                window: window.clone(),
            },
            // match any other event
            _ => Event::Other {
                timestamp,
                name: format!("{:?}", event),
            },
        };

        Ok(data)
    }
}

/// Receives physical input events.
#[derive(Debug)]
#[pyclass(unsendable)]
pub struct EventReceiver {
    pub(crate) receiver: async_broadcast::Receiver<Event>,
}

/// Contains a vector of events.
#[derive(Debug, Clone)]
#[pyclass]
pub struct EventVec(Vec<Event>);

// convenience methods for KeyEventVec
impl EventVec {
    /// Check if the given KeyEventVec contains the provided key in the
    /// `Pressed` state (convenience method).
    pub fn key_pressed(&self, key: &str) -> bool {
        self.iter().any(|key_event| key_event.key_pressed(key))
    }

    /// Check if the given KeyEventVec contains the provided key in the
    /// `Released` state (convenience method).
    pub fn key_released(&self, key: &str) -> bool {
        self.iter().any(|key_event| key_event.key_released(key))
    }
}

#[pymethods]
impl EventVec {
    /// Check if the given KeyEventVec contains the provided key in the
    /// `Pressed` state (convenience method).
    #[pyo3(name = "key_pressed")]
    pub fn py_key_pressed(&self, key: &str) -> bool {
        self.key_pressed(key)
    }

    /// Check if the given KeyEventVec contains the provided key in the
    /// `Released` state (convenience method).
    #[pyo3(name = "key_released")]
    pub fn py_key_released(&self, key: &str) -> bool {
        self.key_released(key)
    }

    /// Returns all pressed keys in the KeyEventVec.
    #[pyo3(name = "keys_pressed")]
    pub fn py_keys_pressed(&self) -> Vec<String> {
        self.iter()
            .filter_map(|event| match event {
                Event::KeyPress { key, .. } => Some(key.clone()),
                _ => None,
            })
            .collect()
    }

    /// Returns all released keys in the KeyEventVec.
    #[pyo3(name = "keys_released")]
    pub fn py_keys_released(&self) -> Vec<String> {
        self.iter()
            .filter_map(|event| match event {
                Event::KeyRelease { key, .. } => Some(key.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn events(&self) -> Vec<Event> {
        self.0.clone()
    }
}

// make KeyEventVec behave like a vector of KeyEvents
impl Deref for EventVec {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EventReceiver {
    pub fn poll(&mut self) -> EventVec {
        let mut inputs = Vec::new();
        while let Ok(input) = self.receiver.try_recv() {
            inputs.push(input);
        }
        EventVec(inputs)
    }

    /// Flushes the internal buffer of key events for this receiver without
    /// returning them. This is slightly more efficient than calling
    /// `get_keys` and ignoring the result.
    pub fn flush(&mut self) {
        while let Ok(_) = self.receiver.try_recv() {}
    }
}

#[pymethods]
impl EventReceiver {
    /// Polls the receiver for new events.
    #[pyo3(name = "poll")]
    pub fn py_poll(&mut self) -> EventVec {
        self.poll()
    }

    /// Flushes the internal buffer of key events for this receiver without
    /// returning them. This is slightly more efficient than calling
    /// `poll` and ignoring the result.
    #[pyo3(name = "flush")]
    pub fn py_flush(&mut self) {
        self.flush()
    }
}

pub(crate) type EventHandlerId = usize;

pub(crate) type EventHandler = Arc<dyn Fn(Event) -> bool + Send + Sync>;
/// Extension for tvpes
pub trait EventHandlingExt {
    /// Add an event handler to handle a specific event type.
    fn add_event_handler<F>(&self, kind: EventKind, handler: F) -> EventHandlerId
    where
        F: Fn(Event) -> bool + 'static + Send + Sync;

    /// Remove an event handler.
    fn remove_event_handler(&self, id: EventHandlerId);

    /// Dispatch an event. Returns a boolean indicating whether the event was
    /// consumed by any of the handlers.
    fn dispatch_event(&self, event: Event) -> bool;
}

impl FromPyObject<'_> for EventKind {
    fn extract_bound(ob: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let kind = ob.extract::<String>()?;
        // try to convert the string to an EventKind
        let kind = EventKind::from_str(&kind).map_err(|_| {
            let possible_kinds = EventKind::VARIANTS
                .iter()
                .map(|v| format!("{:}", v))
                .collect::<Vec<_>>()
                .join(", ");
            pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid event kind '{:}'. Possible values are: {:}",
                kind, possible_kinds
            ))
        })?;

        Ok(kind)
    }
}
