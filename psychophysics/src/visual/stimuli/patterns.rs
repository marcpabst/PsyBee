// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    utils::AtomicExt,
    visual::{color::RawRgba, geometry::Size, Window},
};

use super::pattern_stimulus::Pattern;

#[derive(Clone, Debug)]
pub struct SineGratings {
    phase: f32,
    cycle_length: Size,
    color: RawRgba,
}

impl SineGratings {
    pub fn new(phase: f32, cycle_length: Size, color: RawRgba) -> Self {
        Self {
            phase,
            cycle_length,
            color,
        }
    }
}

impl Pattern for SineGratings {
    fn get_pattern_function_code(&self, variable_name: &str) -> String {
        format!("
        let frequency = 1.0 / {variable_name}.cycle_length;
        let pos_org = vec4<f32>(in.position_org.xy, 0., 0.);
        var c = sin(frequency * pos_org.x + {variable_name}.phase);
        return vec4<f32>({variable_name}.color.r * c, {variable_name}.color.g * c, {variable_name}.color.b * c, {variable_name}.color.a);
        ")
    }

    fn get_uniform_buffers_data(&self, window: &Window) -> Vec<Vec<u8>> {
        let screen_width_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let screen_width_px = window.width_px.load_relaxed();
        let screen_height_px = window.height_px.load_relaxed();

        // turn cycle_length into a float (in pixels)
        let cycle_length: f32 = self.cycle_length.to_pixels(
            screen_width_mm,
            viewing_distance_mm,
            screen_width_px,
            screen_height_px,
        ) as f32;
        // return the uniform buffer data as a byte slice
        let data1 = [self.phase.to_ne_bytes(), cycle_length.to_ne_bytes()].concat();
        let data2 = self.color.to_ne_bytes().to_vec();
        // 8 bytes of padding to align the data with 32 bytes
        let padding = vec![0; 8];
        vec![[data1, padding, data2].concat()]
    }

    fn get_uniform_struct_code(&self, struct_name: &str) -> String {
        format!(
            "
        struct {struct_name} {{
            phase: f32,
            cycle_length: f32,
            color: vec4<f32>,
        }};
        "
        )
    }
}

// #[derive(Clone, Debug)]
// pub struct GaussianAlphamask<P: Pattern> {
//     /// Standard deviation of the Gaussian for the x direction
//     pub sigma_x: f32,
//     /// Standard deviation of the Gaussian for the y direction
//     pub sigma_y: f32,
//     /// The base pattert
//     pub pattern: P,
// }

// impl Pattern for GaussianAlphamask<SineGratings> {
//     fn get_pattern_function_code(&self, variable_name: &str) -> String {
//         // call the base pattern function
//         let new_variable_name = variable_name.to_string() + "_base";
//         let base_pattern_function_code = self.pattern.get_pattern_function_code(variable_name);

//     }

//     fn get_uniform_buffers_data(&self, window: &Window) -> Vec<Vec<u8>> {
//         let screen_width_mm = window.physical_width.load_relaxed();
//         let viewing_distance_mm = window.viewing_distance.load_relaxed();
//         let screen_width_px = window.width_px.load_relaxed();
//         let screen_height_px = window.height_px.load_relaxed();

//         // turn cycle_length into a float (in pixels)
//         let cycle_length: f32 = self.pattern.cycle_length.to_pixels(
//             screen_width_mm,
//             viewing_distance_mm,
//             screen_width_px,
//             screen_height_px,
//         ) as f32;
//         // return the uniform buffer data as a byte slice
//         let data1 = [self.pattern.phase.to_ne_bytes(), cycle_length.to_ne_bytes()].concat();
//         let data2 = self.pattern.color.to_ne_bytes().to_vec();
//         let data3 = [self.sigma_x.to_ne_bytes(), self.sigma_y.to_ne_bytes()].concat();
//         // 4 bytes of padding to align the data with 32 bytes
//         let padding = vec![0; 4];
//         vec![[data1, padding, data2, padding, data3].concat()]
//     }

//     fn get_uniform_struct_code(&self, struct_name: &str) -> String {
//         format!(
//             "
//         struct {struct_name} {{
//             phase: f32,
//             cycle_length: f32,
//             color: vec4<f32>,
//             sigma_x:
