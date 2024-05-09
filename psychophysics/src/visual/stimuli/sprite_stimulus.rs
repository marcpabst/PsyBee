// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::pattern_stimulus::PatternStimulus;
use super::patterns::Sprite;
use crate::generate_assessors;
use crate::visual::color::IntoRawRgba;
use crate::visual::geometry::Size;
use crate::visual::{
    color::RawRgba, geometry::ToVertices, stimuli::base_stimulus::BaseStimulus,
    window::WindowState, Window,
};

pub type SpriteStimulus = PatternStimulus<Sprite>;

impl SpriteStimulus {
    pub fn new_from_spritesheet(
        window: &Window,
        shape: impl ToVertices + 'static,
        sprite_path: &str,
        num_sprites_x: u32,
        num_sprites_y: u32,
    ) -> Self {
        println!("Creating new sprite stimulus");
        PatternStimulus::new_from_pattern(
            window,
            shape,
            Sprite::new_from_spritesheet(sprite_path, num_sprites_x, num_sprites_y)
                .expect("Failed to load spritesheet"),
        )
    }

    pub fn advance_image_index(&mut self) -> () {
        self.pattern.lock().unwrap().advance_image_index()
    }
}
