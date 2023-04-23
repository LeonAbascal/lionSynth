use std::f32::consts::PI;

pub enum WaveShapes {
    Saw,
    Square,
    Pulse(f32),
    Sine,
    Triangle,
}

pub trait OscillatorMath {
    fn sqr(&self) -> Self;
    fn tri(&self) -> Self;
    fn saw(&self) -> Self;
    fn pulse(&self, pwd: f32) -> Self;
}

impl OscillatorMath for f32 {
    fn sqr(&self) -> Self {
        if *self % 2.0 * PI > PI {
            1.0
        } else {
            0.0
        }
    }

    // TODO
    fn tri(&self) -> Self {
        0.0
    }

    // TODO
    fn saw(&self) -> Self {
        0.0
    }

    fn pulse(&self, pwd: f32) -> Self {
        let pwd = pwd % 2.0 * PI;
        if *self % 2.0 * PI > pwd {
            1.0
        } else {
            0.0
        }
    }
}
