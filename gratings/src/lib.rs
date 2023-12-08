use futures_lite::Future;
use input::Key;
use wasm_bindgen::{closure::Closure, JsCast};
use web_time::Duration;

pub mod input;
pub mod visual;
use async_std::task::{self};
pub enum PFutureReturns {
    Duration(Duration),
    Timeout(Duration),
    KeyPress((Key, Duration)),
    NeverReturn,
}

// implement unwrap_duration for Result<PFutureReturns, anyhow::Error>
pub trait UnwrapDuration {
    fn unwrap_duration(self) -> Duration;
    fn is_duration(&self) -> bool;
}
pub trait UnwrapKeyPressAndDuration {
    fn unwrap_key_and_duration(self) -> (Key, Duration);
    fn is_keypress(&self) -> bool;
}

impl UnwrapDuration for Result<PFutureReturns, anyhow::Error> {
    fn unwrap_duration(self) -> Duration {
        match self {
            Ok(PFutureReturns::Duration(d)) => d,
            Ok(PFutureReturns::Timeout(d)) => d,
            Ok(PFutureReturns::KeyPress((_, d))) => d,
            Ok(PFutureReturns::NeverReturn) => {
                panic!("unwrap_duration() called on PFutureReturns::NeverReturn. this should never happen.")
            }
            Err(_) => {
                // panick with error
                panic!("unwrap_duration() called on an Err value.")
            }
        }
    }
    fn is_duration(&self) -> bool {
        match self {
            Ok(PFutureReturns::Duration(_)) => true,
            Ok(PFutureReturns::Timeout(_)) => true,
            _ => false,
        }
    }
}

impl UnwrapKeyPressAndDuration for Result<PFutureReturns, anyhow::Error> {
    fn unwrap_key_and_duration(self) -> (Key, Duration) {
        match self {
            Ok(PFutureReturns::KeyPress((k, d))) => (k, d),
            Ok(PFutureReturns::NeverReturn) => {
                panic!("unwrap_duration() called on PFutureReturns::NeverReturn. this should never happen.")
            }
            Err(_) => {
                // panick with error
                panic!("unwrap_keypress() called on an Err value.")
            }
            _ => {
                panic!("unwrap_keypress() called on a non-keypress value.")
            }
        }
    }
    fn is_keypress(&self) -> bool {
        match self {
            Ok(PFutureReturns::KeyPress(_)) => true,
            _ => false,
        }
    }
}

pub async fn sleep(secs: f64) -> Result<PFutureReturns, anyhow::Error> {
    let start = web_time::Instant::now();
    async_std::task::sleep(Duration::from_micros((secs * 1000000.0) as u64)).await;
    let end = web_time::Instant::now();
    return Ok(PFutureReturns::Duration(end.duration_since(start)));
}

// macro to log to sdout or console, depending on target
#[macro_export]
macro_rules! log_extra {

    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($arg)*);
        }
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!($($arg)*).into());
        }
    };
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    task::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
