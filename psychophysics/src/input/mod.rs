use std::ops::Deref;
pub use winit::keyboard::KeyCode as Key;
use winit::platform::scancode::PhysicalKeyExtScancode;

use crate::visual::{geometry::Size, Window};

#[derive(Debug, Clone)]
pub struct Event {
    /// The timestamp when the event was received.
    pub timestamp: std::time::SystemTime,
    /// The event data.
    pub data: EventData,
    // /// The winit event (if available) that triggered this input event.
    //pub winit_event: Option<winit::event::WindowEvent>,
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

/// A mouse button.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
    /// The forward mouse button.
    Forward,
    /// The back mouse button.
    Back,
    /// An additional mouse button with the given index.
    Other(u16),
}

#[derive(Debug, Clone)]
pub enum EventData {
    /// A keypress event. This is triggered when a key is pressed.
    KeyPress {
        /// String representation of the key that was pressed.
        key: String,
        /// KeyCode of the key that was pressed.
        code: u32,
    },
    /// A key release event. This is triggered when a key is released.
    KeyRelease {
        /// String representation of the key that was released.
        key: String,
        /// KeyCode of the key that was released.
        code: u32,
    },
    /// A mouse button press event. This is triggered when a mouse button is pressed.
    MouseButtonPress {
        /// The button that was pressed.
        button: MouseButton,
        /// The position of the mouse cursor when the button was pressed.
        position: (Size, Size),
    },
    /// A mouse button release event. This is triggered when a mouse button is released.
    MouseButtonRelease {
        /// The button that was released.
        button: MouseButton,
        /// The position of the mouse cursor when the button was released.
        position: (Size, Size),
    },
    /// A touch event. This is triggered when a touch screen is touched.
    TouchPress {
        /// The position of the touch.
        position: (Size, Size),
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// A touch release event. This is triggered when a touch screen is released.
    TouchRelease {
        /// The position of the touch.
        position: (Size, Size),
        /// The id of the touch (if available).
        id: Option<u64>,
    },
    /// The window has lost focus.
    FocusGained,
    /// The window has gained focus.
    FocusLost,
    /// The mouse cursor was moved.
    CursorMoved {
        /// The position of the cursor.
        position: (Size, Size),
    },
    /// The mouse cursor was entered into the window.
    CursorEntered,
    /// The mouse cursor was exited from the window.
    CursorExited,
    /// A pressure-sensitive touchpad was pressed (if available).
    TouchpadPress {
        /// The pressure of the touch.
        pressure: f32,
        /// The level of the touch.
        stage: i64,
    },
    /// The mouse wheel was scrolled (or the equivalent touchpad gesture).
    MouseWheel {
        /// The amount of horizontal scrolling.
        horizontal: f32,
        /// The amount of vertical scrolling.
        vertical: f32,
    },
    /// Any other event. The string contains the name of the event.
    Other(String),
}

// Implements convenience methods for Event.
impl Event {
    /// Returns true if this element represents a press of the given key.
    pub fn key_pressed(&self, key: &str) -> bool {
        matches!(&self.data, EventData::KeyPress { key: k, .. } if k == key)
    }

    /// Returns true if this element represents a release of the given key.
    pub fn key_released(&self, key: &str) -> bool {
        matches!(&self.data, EventData::KeyRelease { key: k, .. } if k == key)
    }

    /// Returns true if this element represents a press of the given mouse button.
    pub fn mouse_button_pressed(&self, button_a: MouseButton) -> bool {
        matches!(&self.data, EventData::MouseButtonPress { button, .. } if button_a == *button)
    }

    /// Returns true if this element represents a release of the given mouse button.
    pub fn mouse_button_released(&self, button_a: MouseButton) -> bool {
        matches!(&self.data, EventData::MouseButtonRelease { button, .. } if button_a == *button)
    }
}

// fn compare_mouse_button(
//     button_a: MouseButton,
//     button_b: winit::event::MouseButton,
// ) -> bool {
//     match button_b {
//         winit::event::MouseButton::Left => button_a == MouseButton::Left,
//         winit::event::MouseButton::Right => button_a == MouseButton::Right,
//         winit::event::MouseButton::Middle => button_a == MouseButton::Middle,
//         winit::event::MouseButton::Forward => button_a == MouseButton::Forward,
//         winit::event::MouseButton::Back => button_a == MouseButton::Back,
//         winit::event::MouseButton::Other(id) => button_a == MouseButton::Other(id),
//     }
// }

/// Convert a winit WindowEvent to an InputEvent.
impl TryFrom<winit::event::WindowEvent> for Event {
    type Error = &'static str;

    fn try_from(event: winit::event::WindowEvent) -> Result<Self, Self::Error> {
        let timestamp = std::time::SystemTime::now();
        let data = match event {
            // match keyboad events
            winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                ..
            } => {
                let key_str = event.logical_key.to_text().unwrap_or_default();
                let key_code = event.physical_key.to_scancode().unwrap_or_default();

                match event.state {
                    winit::event::ElementState::Pressed => EventData::KeyPress {
                        key: key_str.to_string(),
                        code: key_code,
                    },
                    winit::event::ElementState::Released => EventData::KeyRelease {
                        key: key_str.to_string(),
                        code: key_code,
                    },
                }
            }
            // match mouse button events
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Forward => MouseButton::Forward,
                    winit::event::MouseButton::Back => MouseButton::Back,
                    winit::event::MouseButton::Other(id) => MouseButton::Other(id),
                };
                let position = (Size::Pixels(0.0), Size::Pixels(0.0));
                match state {
                    winit::event::ElementState::Pressed => {
                        EventData::MouseButtonPress { button, position }
                    }
                    winit::event::ElementState::Released => {
                        EventData::MouseButtonRelease { button, position }
                    }
                }
            }
            // match touch events
            winit::event::WindowEvent::Touch(touch) => {
                let _position = (
                    Size::Pixels(touch.location.x),
                    Size::Pixels(touch.location.y),
                );

                EventData::Other("touch".to_string())
            }
            // match focus events
            winit::event::WindowEvent::Focused(focused) => {
                if focused {
                    EventData::FocusGained
                } else {
                    EventData::FocusLost
                }
            }
            // match cursor movement events
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let position = (
                    Size::Pixels(position.x) - Size::ScreenWidth(0.5),
                    Size::Pixels(-position.y) + Size::ScreenHeight(0.5),
                );
                EventData::CursorMoved { position }
            }
            // match cursor enter events
            winit::event::WindowEvent::CursorEntered { .. } => EventData::CursorEntered,
            // match cursor exit events
            winit::event::WindowEvent::CursorLeft { .. } => EventData::CursorExited,
            // match touchpad press events
            winit::event::WindowEvent::TouchpadPressure {
                device_id: _,
                pressure,
                stage,
            } => EventData::TouchpadPress {
                pressure,
                stage: stage,
            },
            // match any other event
            _ => EventData::Other(format!("{:?}", event)),
        };

        Ok(Event {
            timestamp,
            data,
            //winit_event: Some(winit::event::Event::WindowEvent(event)),
        })
    }
}

/// Receives physical input events.
#[derive(Debug)]
pub struct EventReceiver {
    pub(crate) receiver: async_broadcast::Receiver<Event>,
}

/// Contains a vector of events.
#[derive(Debug, Clone)]
pub struct EventVec(Vec<Event>);

// convenience methods for KeyEventVec
impl EventVec {
    /// Check if the given KeyEventVec contains the provided key in the `Pressed` state (convenience method).
    pub fn key_pressed(&self, key: &str) -> bool {
        self.iter().any(|key_event| key_event.key_pressed(key))
    }

    /// Check if the given KeyEventVec contains the provided key in the `Released` state (convenience method).
    pub fn key_released(&self, key: &str) -> bool {
        self.iter().any(|key_event| key_event.key_released(key))
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
    /// Create a new EventReceiver from the given window.
    #[deprecated(note = "use the window's `create_event_receiver` method")]
    pub fn new(window: &Window) -> Self {
        Self {
            receiver: window.event_receiver.activate_cloned(),
        }
    }

    pub fn events(&mut self) -> EventVec {
        let mut inputs = Vec::new();
        while let Ok(input) = self.receiver.try_recv() {
            inputs.push(input);
        }
        EventVec(inputs)
    }

    /// Flushes the internal buffer of key events for this receiver without returning them.
    /// This is slightly more efficient than calling `get_keys` and ignoring the result.
    pub fn flush(&mut self) {
        while let Ok(_) = self.receiver.try_recv() {}
    }
}
