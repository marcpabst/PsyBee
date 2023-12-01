extern crate gratings;
use gratings::visual::gratings::GratingsStimulus;
use gratings::visual::text::TextStimulus;
use gratings::visual::Frame;
use gratings::visual::Screen;
use gratings::visual::Window;

fn main() {
    // define Pi
    let pi = std::f32::consts::PI;

    // create a window
    let mut window = Window::new();

    // create a few stimuli
    let gratings = GratingsStimulus::new(&window, 100.0, 0.0);
    let text = TextStimulus::new(&window, "Hello World!".to_string());

    // create a screen
    let screen = Screen::new(move |frame: &mut Frame| {
        // the code in this closure will be executed on every frame
        gratings.params.lock().unwrap().phase += pi;
        // add the gratings stimulus to the frame
        frame.add(&gratings);
        // add the text stimulus to the frame
        frame.add(&text);
    });

    // show the screen
    window.show(screen);
}
