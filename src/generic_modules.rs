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

pub struct Oscillator {
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

impl Module for Oscillator {
    fn behaviour(&self, _in_data: f32) -> f32 {
        let freq: f32 = 440.0; // TODO: parameterizable
        let amplitude: f32 = 1.0; // TODO: parameterizable gain
        let phase: f32 = 0.0;
        (self.clock * freq * 2.0 * PI * amplitude + phase / self.sample_rate).sin()
        // TODO: parameterizable wave
    }

    fn tick(&mut self) {
        self.clock = (self.clock + 1.0) % self.sample_rate;
    }

    fn get_clock(&self) -> f32 {
        self.clock
    }
}

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

impl Oscillator {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            clock: 0.0,
            sample_rate: sample_rate as f32,
        }
    }
}
