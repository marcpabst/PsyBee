use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use derive_debug::Dbg;
use pyo3::{
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyDict, PyList, PyListMethods, PySequenceMethods, PyTuple, PyTupleMethods},
    IntoPy, Py, PyAny, PyResult, Python,
};
use renderer::{cosmic_text, renderer::RendererFactory};
use winit::event_loop::EventLoopProxy;

use crate::{app::App, errors, visual::window::Window};

#[derive(Dbg)]
pub enum EventLoopAction {
    CreateNewWindow(WindowOptions, Sender<Window>),
    GetAvailableMonitors(Sender<Vec<Monitor>>),
}

#[derive(Debug)]
#[pyclass]
pub struct PyRendererFactory(pub Box<dyn RendererFactory>);

// impl Clone for PyRendererFactory
impl Clone for PyRendererFactory {
    fn clone(&self) -> Self {
        Self(self.0.cloned())
    }
}

// deref for PyRendererFactory
impl std::ops::Deref for PyRendererFactory {
    type Target = dyn RendererFactory;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl PyRendererFactory {
    pub fn inner(&self) -> &dyn RendererFactory {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass]
pub struct Monitor {
    #[pyo3(get)]
    pub name: String,
    pub resolution: (u32, u32),
    handle: winit::monitor::MonitorHandle,
}

impl Monitor {
    pub fn new(name: String, resolution: (u32, u32), handle: winit::monitor::MonitorHandle) -> Self {
        Self {
            name,
            resolution,
            handle,
        }
    }

    pub fn handle(&self) -> &winit::monitor::MonitorHandle {
        &self.handle
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Options for creating a window. The ExperimentManager will try to find a
/// video mode that satisfies the provided constraints. See documentation of the
/// variants for more information.
#[derive(Debug, Clone, PartialEq)]
#[pyclass]
pub enum WindowOptions {
    Windowed {
        /// The width and height of the window in pixels. Defaults to 800x600
        /// (px).
        resolution: Option<(u32, u32)>,
    },
    /// Match the given constraints exactly. You can set any of the constraints
    /// to `None` to use the default value.
    FullscreenExact {
        /// The monitor to use. Defaults to the primary monitor.
        monitor: Option<Monitor>,
        /// The width and height of the window in pixels. Defaults to the width
        /// of the first supported video mode of the selected monitor.
        resolution: Option<(u32, u32)>,
        /// The refresh rate to use in Hz. Defaults to the refresh rate of the
        /// first supported video mode of the selected monitor.
        refresh_rate: Option<f64>,
    },
    /// Select window configuration that satisfies the given constraints and has
    /// the highest refresh rate.
    FullscreenHighestRefreshRate {
        monitor: Option<Monitor>,
        resolution: Option<(u32, u32)>,
    },
    /// Select the highest resolution that satisfies the given constraints and
    /// has the highest resolution.
    FullscreenHighestResolution {
        monitor: Option<Monitor>,
        refresh_rate: Option<f64>,
    },
}

impl WindowOptions {
    pub fn monitor(&self) -> Option<&Monitor> {
        match self {
            WindowOptions::Windowed { .. } => None,
            WindowOptions::FullscreenExact { monitor, .. } => monitor.as_ref(),
            WindowOptions::FullscreenHighestRefreshRate { monitor, .. } => monitor.as_ref(),
            WindowOptions::FullscreenHighestResolution { monitor, .. } => monitor.as_ref(),
        }
    }
}

/// The ExperimentManager is available to the user in the experiment function.
#[derive(Debug, Clone)]
#[pyclass(unsendable)]
pub struct ExperimentManager {
    event_loop_proxy: EventLoopProxy<()>,
    action_sender: Sender<EventLoopAction>,
    renderer_factory: Arc<dyn RendererFactory>,
    font_manager: Arc<Mutex<cosmic_text::FontSystem>>,
}

impl ExperimentManager {
    pub fn new(
        event_loop_proxy: EventLoopProxy<()>,
        action_sender: Sender<EventLoopAction>,
        renderer_factory: Arc<dyn RendererFactory>,
        font_manager: Arc<Mutex<cosmic_text::FontSystem>>,
    ) -> Self {
        Self {
            event_loop_proxy,
            action_sender,
            renderer_factory,
            font_manager,
        }
    }

    pub fn font_manager(&self) -> &Arc<Mutex<cosmic_text::FontSystem>> {
        &self.font_manager
    }

    pub fn renderer_factory(&self) -> &Arc<dyn RendererFactory> {
        &self.renderer_factory
    }

    /// Create a new window with the given options. This function will dispatch
    /// a new UserEvent to the event loop and wait until the winit window
    /// has been created. Then it will setup the wgpu device and surface and
    /// return a new Window object.
    pub fn create_window(&self, window_options: &WindowOptions) -> Window {
        // set up window by dispatching a new CreateNewWindow action
        let (sender, receiver) = channel();
        let action = EventLoopAction::CreateNewWindow(window_options.clone(), sender);

        // send action
        println!("Sending action");
        self.action_sender.send(action).unwrap();
        self.event_loop_proxy.send_event(());

        // wait for response
        let window = receiver.recv().expect("Failed to create window");
        log::debug!("New window successfully created");

        window
    }

    /// Create a new window. This is a convenience function that creates a
    /// window with the default options.
    pub fn create_default_window(&self, fullscreen: bool, monitor: Option<u32>) -> Window {
        // select monitor 1 if available
        // find all monitors available

        let monitors = self.get_available_monitors();
        println!("{:?}", monitor);
        println!("Found monitors: {:?}", monitors);
        // get the second monitor if available, otherwise use the first one
        let monitor = monitors
            .get(monitor.unwrap_or(0) as usize)
            .unwrap_or(monitors.first().expect("No monitor found - this should not happen"));

        println!("Creating default window on monitor {:?}", monitor);
        self.create_window(&WindowOptions::FullscreenHighestResolution {
            monitor: Some(monitor.clone()),
            refresh_rate: None,
        })
    }

    /// Retrive available monitors.
    pub fn get_available_monitors(&self) -> Vec<Monitor> {
        let (sender, receiver) = channel();
        self.action_sender
            .send(EventLoopAction::GetAvailableMonitors(sender.clone()))
            .unwrap();

        // wake up the event loop
        self.event_loop_proxy.send_event(());

        println!("waiting for monitors");
        receiver.recv().unwrap()
    }
}

#[pymethods]
impl ExperimentManager {
    #[pyo3(name = "create_default_window")]
    #[pyo3(signature = (fullscreen = false, monitor = None))]
    /// Create a new window. This is a convenience function that creates a
    /// window with the default options.
    ///
    /// Even when `fullscreen` is set to `True`, no video mode changes will be
    /// initiated. The window will be created with the highest resolution
    /// changes. When `fullscreen` is set to `true`,
    /// `monitor` can be used to select the monitor to use. Monitor enumeration
    /// is OS-specific and the primary monitor may not always be at index 0.
    ///
    /// Parameters
    /// ----------
    /// fullscreen : bool, optional
    ///   Whether to create a fullscreen window. Defaults to `false`.
    /// monitor : int, optional
    ///   The index of the monitor to use. Defaults to 0.
    ///
    /// Returns
    /// -------
    /// Window
    ///  The new window.
    fn py_create_default_window(&self, fullscreen: bool, monitor: Option<u32>) -> Window {
        self.create_default_window(fullscreen, monitor)
    }

    #[pyo3(name = "get_available_monitors")]
    fn py_get_available_monitors(&self) -> Vec<Monitor> {
        self.get_available_monitors()
    }
}

/// Runs your experiment function. This function will block the current thread
/// until the experiment function returns!
///
/// Parameters
/// ----------
/// experiment_fn : callable
///    The function that runs your experiment. This function should take a single argument, an instance of `ExperimentManager`, and should not return nothing.
#[pyfunction]
#[pyo3(name = "run_experiment", signature = (py_experiment_fn, *args, **kwargs))]
pub fn py_run_experiment(
    py: Python,
    py_experiment_fn: Py<PyAny>,
    args: Py<PyTuple>,
    kwargs: Option<Py<PyDict>>,
) -> PyResult<()> {
    // create app
    let mut app = App::new();

    // set the __globals__ to make "_renderer_factory" available
    // this will allow functions to create renderer-specific objects
    // without having to pass the renderer object around

    let globals = PyDict::new(py);
    let renderer_factory = PyRendererFactory(app.renderer_factory.cloned());

    py_experiment_fn
        .getattr(py, "__globals__")?
        .bind(py)
        .downcast::<PyDict>()?
        .set_item("__renderer_factory", renderer_factory)?;

    let rust_experiment_fn = move |em: ExperimentManager| -> Result<(), errors::psydkError> {
        Python::with_gil(|py| -> _ {
            // bind kwargs
            let kwargs = if let Some(kwargs) = kwargs {
                kwargs.into_bound(py)
            } else {
                PyDict::new(py)
            };

            py_experiment_fn
                .getattr(py, "__globals__")?
                .bind(py)
                .downcast::<PyDict>()?
                .set_item("__experiment_manager", em.clone())?;

            // TODO: There must be a better way to do this!
            let args = args.bind(py);
            let args_as_seq = args.to_list();
            let args_as_seq = args_as_seq.as_sequence();
            let em = em.into_py(py);
            let em_as_seq = PyList::new_bound(py, vec![em]);
            let em_as_seq = em_as_seq.as_sequence();

            let args = em_as_seq.concat(args_as_seq).unwrap();
            let args = args.to_tuple().unwrap();

            py_experiment_fn.call_bound(py, args, Some(&kwargs))
        })?;
        Ok(())
    };

    py.allow_threads(move || app.run_experiment(rust_experiment_fn))?; // run the experiment
    println!("Experiment finished");
    Ok(())
}
