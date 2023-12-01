fn main() {
    // create a window
    let window = create_default_window(True);

    // create fixation cross
    let fixation_cross = FixationCrossStimulus::new(&window, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5);

    // create a screen
    let fixation_screen = Screen::new(&window, |time: i32| -> i32 {
        println!("Fixation screen: {}", time);
    });

    // show the screen
    window.show_screen(fixation_screen);

    // keep thread alive until window is closed
    window.wait_for_close();
}
