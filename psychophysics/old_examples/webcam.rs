use std::sync::Arc;

use facetracking::face_landmarks;
use psychophysics::camera;
use psychophysics::prelude::*;

use facetracking::face_detection::model_blazeface::BlazefaceModel;
use facetracking::face_detection::FaceDetectionModel;
use facetracking::face_landmarks::model_mediapipe::MediapipeFaceLandmarksModel;
use facetracking::face_landmarks::FaceLandmarksModel;
use facetracking::utils::{SharedState, State};

use imageproc::drawing::{draw_cross_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;

fn show_image(
    window: Window,
) -> Result<(), psychophysics::errors::PsychophysicsError> {
    // create a 640, height: 480 image
    let thatcher = image::DynamicImage::new_rgb8(640, 480);

    // create image stimulus
    let mut image_stim = ImageStimulus::new(
        &window,
        thatcher,
        Rectangle::new(
            Size::ScreenWidth(-0.4),
            Size::ScreenHeight(-0.4),
            Size::ScreenWidth(0.8),
            Size::ScreenHeight(0.8),
        ),
    );

    // when we pass the image to the thread, we need to clone it
    // so that we can still use it in the main thread - don't worry,
    // all stimuli can be cloned and will take care of the underlying
    // data synchronization for you
    let image_stim_clone = image_stim.clone();

    // spawn new thread
    let thread = std::thread::spawn(|| {
        // create face detection model
        let face_detection_model = BlazefaceModel::new();
        let face_landmarks_model = MediapipeFaceLandmarksModel::new();

        // list camras
        let camera_manager = camera::CameraManager::new();
        let cameras = camera_manager.cameras();
        // select first cameraDelphi method
        let camera = cameras.first().unwrap();
        // select first mode
        let mode = &camera.modes()[18];
        // open camera
        let stream = camera.open_with_callback(mode, move |frame| {
            let image: image::RgbImage = frame.into();

            let mut dimage = image::DynamicImage::ImageRgb8(image);

            // flip image
            dimage = dimage.fliph();

            // detect face
            let face_bbox = face_detection_model.run(&dimage);
            let face_bbox_rect = Rect::at(
                face_bbox.origin().0 as i32,
                face_bbox.origin().1 as i32,
            )
            .of_size(
                face_bbox.width() as u32,
                face_bbox.height() as u32,
            );

            // detect landmarks
            let (landmarks, _) = face_landmarks_model
                .run(&dimage, Some(face_bbox.to_tuple()));

            let face_landmarks = landmarks.get_landmarks();

            for landmark in face_landmarks {
                draw_cross_mut(
                    &mut dimage,
                    image::Rgba([0, 255, 0, 1]),
                    landmark.x as i32,
                    landmark.y as i32,
                );
            }

            // plot face bbox on image
            draw_hollow_rect_mut(
                &mut dimage,
                face_bbox_rect,
                image::Rgba([255, 0, 0, 1]),
            );

            // update image stimulus
            image_stim_clone.set_image(dimage);
        });
    });

    // show frames until space key is pressed
    loop_frames!(frame from window, keys = Key::Space, {
        // set frame color to white
        frame.set_bg_color(color::WHITE);
        // add stimuli to frame
        frame.add(&image_stim);
    });

    // close window
    window.close();

    Ok(())
}

fn main() {
    // run experiment
    start_experiment(show_image);
}
