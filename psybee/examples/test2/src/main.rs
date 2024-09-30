use psybee_proc::StimulusParams;

pub enum StimulusParam {
    Size {
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    Float {
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
}

#[derive(StimulusParams, Clone, Debug)]
pub struct GaborParams {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub cycle_length: f64,
    pub phase: f64,
    pub sigma: f64,
    pub orientation: f64,
}

fn main() {
    let params = GaborParams {
        cx: 0.0,
        cy: 0.0,
        radius: 1.0,
        cycle_length: 1.0,
        phase: 0.0,
        sigma: 1.0,
        orientation: 0.0,
    };

    let fields = params.get_params();
    println!("{:?}", fields);
}
