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
use super::patterns::Image;



use crate::visual::{
    geometry::ToVertices, Window,
};

pub type ImageStimulus = PatternStimulus<Image>;

impl ImageStimulus {
    pub fn new(
        window: &Window,
        shape: impl ToVertices + 'static,
        image_path: &str,
    ) -> Self {
        PatternStimulus::new_from_pattern(
            window,
            shape,
            Image::new_from_path(image_path).expect("Failed to load image"),
        )
    }
}
