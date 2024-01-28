// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use psychophysics::camera;

fn main() {
    // spawn new thread
    let thread = std::thread::spawn(|| {
        // list camras
        let camera_manager = camera::CameraManager::new();
        let cameras = camera_manager.cameras();
        // select first camera
        let camera = cameras.first().unwrap();
        // print all modes
        for (i, mode) in camera.modes().iter().enumerate() {
            println!("mode {}: {:?}", i, mode);
        }
        // select first mode
        let mode = &camera.modes()[10];
        // print mode
        println!("selected mode: {:?}", mode);
        // open camera
        let stream = camera.open_with_callback(mode, |frame| {
            let image: image::RgbImage = frame.into();
        });
    });

    // wait for thread to finish
    thread.join().unwrap();
}
