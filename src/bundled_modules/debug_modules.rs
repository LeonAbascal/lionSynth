use crate::module::{Module, Parameter};
use std::f32::consts::PI;

// MODULES
pub struct PassTrough {
    parameters: Vec<Parameter>,
}

pub struct OscDebug {
    sample_rate: f32,
    parameters: Vec<Parameter>,
}

// IMPLEMENTATIONS
impl Module for PassTrough {
    fn behaviour(&self, in_sample: f32, _time: f32) -> f32 {
        in_sample // clean data
    }

    fn get_parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn get_parameters_mutable(&mut self) -> &mut Vec<Parameter> {
        &mut self.parameters
    }

    fn get_name(&self) -> String {
        "PassThrough".to_string()
    }
}

impl Module for OscDebug {
    fn behaviour(&self, _: f32, time: f32) -> f32 {
        let freq: f32 = 440.0;
        (time * freq * 2.0 * PI / self.sample_rate).sin()
    }

    fn get_parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn get_parameters_mutable(&mut self) -> &mut Vec<Parameter> {
        &mut self.parameters
    }

    fn get_name(&self) -> String {
        "Debug oscillator".to_string()
    }
}

// CONSTRUCTORS
impl PassTrough {
    pub fn new() -> Self {
        Self { parameters: vec![] }
    }
}

impl OscDebug {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            parameters: vec![],
        }
    }
}

// TESTS
#[cfg(test)]
mod test {
    use super::*;
    use crate::SAMPLE_RATE;

    #[test]
    fn test_debug_osc() {
        let mut tested_module = OscDebug::new(SAMPLE_RATE);
        let mut buffer: Vec<f32> = vec![0.0; 10];

        tested_module.fill_buffer(&mut buffer);

        let deterministic_buffer: Vec<f32> = vec![
            0.0,
            0.062648326,
            0.12505053,
            0.18696144,
            0.24813786,
            0.3083394,
            0.3673296,
            0.42487666,
            0.48075455,
            0.53474367,
        ];

        assert_eq!(deterministic_buffer, buffer);
    }

    #[test]
    fn test_pass_through() {
        let mut tested_module = PassTrough::new();
        let mut osc = OscDebug::new(SAMPLE_RATE);
        let mut original_buffer: Vec<f32> = vec![0.0; 20];

        // MODIFY THE BUFFER
        osc.fill_buffer(&mut original_buffer);
        let mut modified_buffer = original_buffer.clone();

        tested_module.fill_buffer(&mut modified_buffer);
        assert_eq!(original_buffer, modified_buffer);
    }
}
