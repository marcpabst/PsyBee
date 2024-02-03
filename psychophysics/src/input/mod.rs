use std::ops::Deref;
pub use winit::keyboard::KeyCode as Key;

use crate::visual::Window;

/// A high-level input physical event. This is an abstraction over the winit input events,
/// which are extremely powerful but also quite complex. This enum is used to
/// represent the most common input events that are used in psychophysics experiments,
/// such as key presses, and touch events.
///
/// The events are divided into two categories: virtual events and device events.
/// Virtual events are triggered by events on the window, e.g. a mouse click INSIDE the window.
/// Device events are triggered regardless of the window, e.g. a mouse click OUTSIDE the window.
/// Note that many real events can trigger both a virtual and a device event.
///
/// If you need more control over the input events, you can listen for winit window or
/// device events directly.
#[derive(Debug, Clone)]
pub enum PhysicalInput {
    // virtual events
    /// A keyboard event. This is triggered when a key is pressed or released.
    KeyInput(winit::event::WindowEvent),
    /// A mouse button event. This is triggered when a mouse button is pressed or released.
    MouseButtonInput(winit::event::WindowEvent),
    /// A mouse wheel event. This is triggered when the mouse wheel is scrolled.
    MouseWheelInput(winit::event::WindowEvent),
    /// A cursor movement event. This is triggered when the mouse cursor is moved.
    /// Note that this cotains the cursor position in the window (not the delta movement).
    /// You should also be aware that depending on the system settings, the operating system
    /// may apply some transformations to the cursor movement, e.g. acceleration.
    CursorMovementInput(winit::event::WindowEvent),
    /// A touch event. This is triggered when a touch screen is touched.
    TouchInput(winit::event::WindowEvent),

    // device events
    /// A raw key event. This is triggered when a key is pressed or released even if the window is not focused.
    RawKeyInput(winit::event::DeviceEvent),
    /// A raw mouse button event. This is triggered when a mouse button is pressed or released even if the window is not focused.
    RawMouseButtonInput(winit::event::DeviceEvent),
    /// A raw mouse wheel event. This is triggered when the mouse wheel is scrolled even if the window is not focused.
    RawMouseWheelInput(winit::event::DeviceEvent),
    /// A raw mouse movement event. This is triggered when the mouse cursor is moved even if the window is not focused.
    /// Note that this contains the delta movement of the mouse cursor.
    ///
    /// Note that even if you start from the same position, summing up the delta movements will not necessarily
    /// give you the same position as the cursor. This is because the operating systems usually apply some transformations
    /// like acceleration to the mouse movement.
    RawMouseMovementInput(winit::event::DeviceEvent),
}

impl PhysicalInput {
    /// Returns true if this element represents a press of the given key.
    pub fn key_pressed(&self, key: Key) -> bool {
        matches!(
            self,
            PhysicalInput::KeyInput(winit::event::WindowEvent::KeyboardInput { event, .. })
                if event.state == winit::event::ElementState::Pressed && event.physical_key == winit::keyboard::PhysicalKey::Code(key)
        )
    }

    /// Returns true if this element represents a release of the given key.
    pub fn key_released(&self, key: Key) -> bool {
        matches!(
            self,
            PhysicalInput::KeyInput(winit::event::WindowEvent::KeyboardInput { event, .. })
                if event.state == winit::event::ElementState::Released && event.physical_key == winit::keyboard::PhysicalKey::Code(key)
        )
    }

    /// try to convert a winit window event to a PhysicalInput
    /// returns None if the event is not a PhysicalInput
    pub fn from_window_event(
        event: winit::event::WindowEvent,
    ) -> Option<Self> {
        match event {
            winit::event::WindowEvent::KeyboardInput { .. } => {
                Some(PhysicalInput::KeyInput(event))
            }
            winit::event::WindowEvent::MouseWheel { .. } => {
                Some(PhysicalInput::MouseWheelInput(event))
            }
            winit::event::WindowEvent::CursorMoved { .. } => {
                Some(PhysicalInput::CursorMovementInput(event))
            }
            winit::event::WindowEvent::MouseInput { .. } => {
                Some(PhysicalInput::MouseButtonInput(event))
            }
            winit::event::WindowEvent::Touch(touch) => {
                Some(PhysicalInput::TouchInput(event))
            }
            _ => None,
        }
    }

    /// try to convert a winit device event to a PhysicalInput
    /// returns None if the event is not a PhysicalInput
    pub fn from_device_event(
        event: winit::event::DeviceEvent,
    ) -> Option<Self> {
        match event {
            winit::event::DeviceEvent::Key(ref _key) => {
                Some(PhysicalInput::RawKeyInput(event))
            }
            winit::event::DeviceEvent::MouseWheel { .. } => {
                Some(PhysicalInput::RawMouseWheelInput(event))
            }
            winit::event::DeviceEvent::Motion { .. } => {
                Some(PhysicalInput::RawMouseMovementInput(event))
            }
            winit::event::DeviceEvent::Button { .. } => {
                Some(PhysicalInput::RawMouseButtonInput(event))
            }
            _ => None,
        }
    }
}

/// Receives physical input events.
pub struct PhysicalInputReceiver {
    receiver: async_broadcast::Receiver<PhysicalInput>,
}

/// Contains a vector of key events. This is returned by the `get_keys` method of the `KeyPressReceiver`.
/// Implements `Deref` so that it can be used as a vector of `KeyEvent`s.
#[derive(Debug, Clone)]
pub struct PhysicalInputVec(Vec<PhysicalInput>);

// convenience methods for PhysicalInputVec
impl PhysicalInputVec {
    /// Check if the given KeyEventVec contains the provided key in the `Pressed` state (convenience method).
    pub fn key_pressed(&self, key: Key) -> bool {
        self.iter().any(|key_event| key_event.key_pressed(key))
    }

    /// Check if the given KeyEventVec contains the provided key in the `Released` state (convenience method).
    pub fn key_released(&self, key: Key) -> bool {
        self.iter().any(|key_event| key_event.key_released(key))
    }
}

// make KeyEventVec behave like a vector of KeyEvents
impl Deref for PhysicalInputVec {
    type Target = Vec<PhysicalInput>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PhysicalInputReceiver {
    /// Create a new KeyPressReceiver for the given window.
    pub fn new(window: &Window) -> Self {
        Self {
            receiver: window
                .physical_input_receiver
                .activate_cloned(),
        }
    }

    pub fn get_inputs(&mut self) -> PhysicalInputVec {
        let mut inputs = Vec::new();
        while let Ok(input) = self.receiver.try_recv() {
            inputs.push(input);
        }
        PhysicalInputVec(inputs)
    }

    /// Flushes the internal buffer of key events for this receiver without returning them.
    /// This is slightly more efficient than calling `get_keys` and ignoring the result.
    pub fn flush(&mut self) {
        while let Ok(_) = self.receiver.try_recv() {}
    }
}
