// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ndarray::{s, Array, Axis, IxDyn};
use ort::{GraphOptimizationLevel, Session};

struct ONNXModel<M> {
    model: M,
}

type ORTModel = ONNXModel<ort::Session>;

impl ORTModel {
    pub fn new_from_memory(bytes: &[u8]) -> Result<Self, ort::Error> {
        let model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_memory(bytes)?;

        Ok(Self { model })
    }

    pub fn predict(
        &self,
        inputs: Array<f32, IxDyn>,
    ) -> Result<Vec<Array<f32, IxDyn>>, ort::Error> {
        let inputs = ort::inputs!(inputs)?;
        let outputs = self.model.run(inputs);
        let outputs = outputs?;

        let mut output_vec = Vec::with_capacity(outputs.len());
        for (_, e) in outputs.iter() {
            output_vec.push(e.extract_tensor::<f32>().unwrap().view().to_owned());
        }
        Ok(output_vec)
    }
}
