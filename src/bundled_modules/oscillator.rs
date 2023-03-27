use crate::module::Module;
use std::f32::consts::PI;

pub struct Oscillator {
    clock: f32,
    sample_rate: f32,
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

impl Module for Oscillator {
    fn behaviour(&self, _in_data: f32) -> f32 {
        (self.clock * self.frequency * 2.0 * PI * self.amplitude + self.phase / self.sample_rate)
            .sin()
        // TODO: parameterizable wave, amp, freq, phase
    }

    fn tick(&mut self) {
        self.clock = (self.clock + 1.0) % self.sample_rate;
    }

    fn get_clock(&self) -> f32 {
        self.clock
    }
}

impl Oscillator {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            clock: 0.0,
            sample_rate: sample_rate as f32,
            frequency: 440.0,
            amplitude: 1.0,
            phase: 0.0,
        }
    }
}
