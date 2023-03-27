use crate::module::Module;
use std::f32::consts::PI;

// MODULES
#[cfg(debug_assertions)]
#[allow(dead_code)]
pub struct PassTrough {}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub struct OscDebug {
    clock: f32,
    sample_rate: f32,
}

// IMPLEMENTATIONS
#[cfg(debug_assertions)]
impl Module for PassTrough {
    fn behaviour(&self, in_sample: f32) -> f32 {
        in_sample // clean data
    }

    fn tick(&mut self) {}
    fn get_clock(&self) -> f32 {
        0.0
    }
}

#[cfg(debug_assertions)]
impl Module for OscDebug {
    fn behaviour(&self, _: f32) -> f32 {
        let freq: f32 = 440.0;
        (self.clock * freq * 2.0 * PI / self.sample_rate).sin()
    }

    fn tick(&mut self) {
        self.clock = (self.clock + 1.0) % self.sample_rate;
    }
    fn get_clock(&self) -> f32 {
        self.clock
    }
}

// CONSTRUCTORS
#[cfg(debug_assertions)]
#[allow(dead_code)]
impl PassTrough {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
impl OscDebug {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            clock: 0.0,
            sample_rate: sample_rate as f32,
        }
    }
}

// TESTS
#[cfg(test)]
mod test {
    use super::*;
    use crate::SAMPLE_RATE;

    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_osc() {
        let mut tested_module = OscDebug::new(SAMPLE_RATE);
        let mut buffer: Vec<f32> = vec![0.0; 10];

        tested_module.fill_buffer(&mut buffer);

        let deterministic_buffer: Vec<f32> = vec![
            0.062648326,
            0.12505053,
            0.18696144,
            0.24813786,
            0.3083394,
            0.3673296,
            0.42487666,
            0.48075455,
            0.53474367,
            0.586632,
        ];

        assert_eq!(deterministic_buffer, buffer);
    }

    #[test]
    #[cfg(debug_assertions)]
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
