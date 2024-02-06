// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// # Fill patterns
//
/// This module contains the implementation of various fill patterns that can be used with the `PatternStimulus`.
///
/// When workig with `PatternStimulus`, you will come across three important concepts:
///
/// - `Shape`: The shape of the pattern. This can be a rectangle, a circle, or some other shape.
/// - `FillPattern`: The pattern that fills the shape. This can be e.g. a uniform color, a sine grating, an image, or even a video.
///    Some fill patterns act as wrappers around other fill patterns, e.g. the `GaussianAlphamask` pattern, which modulates the alpha value of another pattern.
/// - `OutlinePattern`: The outline of the shape. This can be a solid color, a gradient, a dashed line, or other patterns.
///
/// The `PatternStimulus` is a combination of these three concepts. It is a shape that is filled with a pattern and has an outline.
/// While both the fill and the outline do not have to be present, the shape is always present.
///
use derive_builder::Builder;

use crate::{
    utils::{AtomicExt, ToWgslBytes},
    visual::{
        color::RawRgba,
        geometry::{Size, SizeVector2D, ToPixels},
        Window,
    },
};

use super::pattern_stimulus::FillPattern;

/// A uniform color pattern
#[derive(Clone, Debug)]
pub struct UniformColor {
    color: RawRgba,
}

impl UniformColor {
    pub fn new(color: RawRgba) -> Self {
        Self { color }
    }
}

impl FillPattern for UniformColor {
    fn get_uniform_buffers_data(&self, _window: &Window) -> Vec<Vec<u8>> {
        vec![self.color.to_ne_bytes().to_vec()]
    }

    fn get_uniform_structs_code(
        &self,
        uuid: &str,
    ) -> Vec<(String, String, String)> {
        vec![(
            format!("struct {uuid}_s1 {{ color: vec4<f32>,}}"),
            format!("{uuid}_s1"),
            format!("{uuid}_0"),
        )]
    }

    fn get_pattern_functions_code(&self, uuid: &str) -> String {
        format!(
            "
            fn {uuid}_uniform_color(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {{
                let params = {uuid}_0;
                return vec4<f32>(params.color.r, params.color.g, params.color.b, params.color.a);
            }}
        ")
    }

    fn get_pattern_expression(&self, uuid: &str) -> String {
        format!("{uuid}_uniform_color(x, y, color)")
    }
}

/// A Sinosoidal grating pattern
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

impl FillPattern for SineGratings {
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
        let data1 =
            [self.phase.to_ne_bytes(), cycle_length.to_ne_bytes()].concat();
        let data2 = self.color.to_ne_bytes().to_vec();
        // 8 bytes of padding to align the data with 32 bytes
        let padding = vec![0; 8];
        vec![[data1, padding, data2].concat()]
    }

    fn get_uniform_structs_code(
        &self,
        uuid: &str,
    ) -> Vec<(String, String, String)> {
        vec![(
            format!("struct {uuid}_s1 {{ phase: f32, cycle_length: f32, color: vec4<f32>,}}"),
            format!("{uuid}_s1"),
            format!("{uuid}_0"),
        )]
    }

    fn get_pattern_functions_code(&self, uuid: &str) -> String {
        format!("
            fn {uuid}_sine_gratings(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {{
                let params = {uuid}_0;
                let frequency = 1.0 / params.cycle_length;
                var c = sin(frequency * x + params.phase);
                return vec4<f32>(params.color.r * c, params.color.g * c, params.color.b * c, params.color.a);
            }}
        ")
    }

    fn get_pattern_expression(&self, uuid: &str) -> String {
        format!("{uuid}_sine_gratings(x, y, color)")
    }
}

/// A Gaussian mask that can be used to modulate the alpha value of a pattern
/// The Gaussian is defined as:
/// ```
/// f(x, y) = 1 / (2 * pi * sigma.x * sigma.y) * exp(-((x - mu.x)^2 / (2 * sigma.x^2) + (y - mu.y)^2 / (2 * sigma.y^2)))
/// ```
///
/// The alpha value of the pattern is then modulated by the Gaussian:
/// ```
/// alpha = f(x, y) / f(0, 0)
/// ```
///
/// The GaussianAlphamask pattern is defined by the following parameters:
/// - `mu.x`: the mean of the Gaussian in the x direction
/// - `mu.y`: the mean of the Gaussian in the y direction
/// - `sigma.x`: the standard deviation of the Gaussian in the x direction
/// - `sigma.y`: the standard deviation of the Gaussian in the y direction
#[derive(Clone, Debug)]
pub struct GaussianAlphamask<P: FillPattern> {
    base_pattern: P,
    mu: SizeVector2D,
    sigma: SizeVector2D,
}

impl<P: FillPattern> GaussianAlphamask<P> {
    pub fn new<M, S>(base_pattern: P, mu: M, sigma: S) -> Self
    where
        M: Into<SizeVector2D>,
        S: Into<SizeVector2D>,
    {
        Self {
            base_pattern,
            mu: mu.into(),
            sigma: sigma.into(),
        }
    }
}

impl<P: FillPattern> FillPattern for GaussianAlphamask<P> {
    fn get_uniform_buffers_data(&self, window: &Window) -> Vec<Vec<u8>> {
        // first, we need to convert the
        // here, we need to combine the uniform buffer data of the base pattern with the data of the GaussianAlphamask
        // we do this by just concatenating the two byte vectors
        let screenwidth_mm = window.physical_width.load_relaxed();
        let viewing_distance_mm = window.viewing_distance.load_relaxed();
        let screenwidth_px = window.width_px.load_relaxed();
        let screenheight_px = window.height_px.load_relaxed();

        let base_data = self.base_pattern.get_uniform_buffers_data(window);

        let (mu_x, mu_y) = self.mu.to_pixels(
            screenwidth_mm,
            viewing_distance_mm,
            screenwidth_px,
            screenheight_px,
        );
        let (sigma_x, sigma_y) = self.sigma.to_pixels(
            screenwidth_mm,
            viewing_distance_mm,
            screenwidth_px,
            screenheight_px,
        );

        log::info!(
            "mu.x: {:?}, mu.y: {:?}, sigma.x: {:?}, sigma.y: {:?}",
            mu_x,
            mu_y,
            sigma_x,
            sigma_y
        );

        let data = vec![vec![
            (mu_x as f32).to_ne_bytes().to_vec(),
            (mu_y as f32).to_ne_bytes().to_vec(),
            (sigma_x as f32).to_ne_bytes().to_vec(),
            (sigma_y as f32).to_ne_bytes().to_vec(),
        ]
        .concat()];

        // print data as float32 (use unsafe to cast to f32)
        log::info!(
            "from bytes: mu.x: {:?}, mu.y: {:?}, sigma.x: {:?}, sigma.y: {:?}",
            f32::from_ne_bytes(data[0][0..4].try_into().unwrap()),
            f32::from_ne_bytes(data[0][4..8].try_into().unwrap()),
            f32::from_ne_bytes(data[0][8..12].try_into().unwrap()),
            f32::from_ne_bytes(data[0][12..16].try_into().unwrap())
        );

        return vec![base_data, data].concat();
    }

    fn get_uniform_structs_code(
        &self,
        uuid: &str,
    ) -> Vec<(String, String, String)> {
        // modify the uuid for the base pattern
        let new_uuid = format!("{uuid}_inner");

        let base_structs =
            self.base_pattern.get_uniform_structs_code(&new_uuid);

        // add our own struct
        let struct_code =
            format!("struct {uuid}_s1 {{ mu: vec2<f32>, sigma: vec2<f32>,}}");

        let tpl = (struct_code, format!("{uuid}_s1"), format!("{uuid}_0"));

        // combine the base structs with our own (order is not important)
        let mut result = base_structs;
        result.push(tpl);
        return result;
    }

    fn get_pattern_functions_code(&self, uuid: &str) -> String {
        let base_functions = self
            .base_pattern
            .get_pattern_functions_code(&format!("{uuid}_inner"));
        format!(
            "
            {base_functions}

            const {uuid}_PI: f32 = 3.14159265359;

            fn {uuid}_gaussian_2d(x: f32, y: f32, mu: vec2<f32>, sigma: vec2<f32>) -> f32 {{
                // Calculate the 2D Gaussian

                // calculate the normalization factor (python: 1. / (2. * np.pi * sx * sy))
                let norm = 1.0 / (2.0 * {uuid}_PI * sigma.x * sigma.y);

                // calculate the exponent (python: np.exp(-((x - mx)**2. / (2. * sx**2.) + (y - my)**2. / (2. * sy**2.))))
                let exponent = -((x - mu.x) * (x - mu.x) / (2.0 * sigma.x * sigma.x) + (y - mu.y) * (y - mu.y) / (2.0 * sigma.y * sigma.y));

                // return the final value
                return norm * exp(exponent);


            }}
            

            fn {uuid}_gaussian_alphamask(x: f32, y: f32, color: vec4<f32>) -> vec4<f32> {{
                // get the parameters
                let params = {uuid}_0;

                // calculate the alpha value
                let mean_val = {uuid}_gaussian_2d(params.mu.x, params.mu.y, params.mu, params.sigma);
                var alpha = {uuid}_gaussian_2d(x, y, params.mu, params.sigma) / mean_val;

                // return the final color
                return vec4<f32>(color.r, color.g, color.b, color.a * alpha);
            }}
        ")
    }

    fn get_pattern_expression(&self, uuid: &str) -> String {
        let base_expression: String = self
            .base_pattern
            .get_pattern_expression(&format!("{uuid}_inner"));
        format!("{uuid}_gaussian_alphamask(x, y, {base_expression})")
    }
}
