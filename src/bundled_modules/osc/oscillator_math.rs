use std::f32::consts::PI;

#[non_exhaustive]
pub enum WaveShape {
    Saw,
    Square,
    Pulse(f32),
    Sine,
    Triangle,
}

impl Default for WaveShape {
    fn default() -> Self {
        WaveShape::Sine
    }
}

fn linear(x: f32, y: f32, m: f32) -> f32 {
    m * x + y
}

pub trait OscillatorMath {
    fn tri(&self) -> Self;
    fn saw(&self) -> Self;
    fn sqr(&self) -> Self;
    fn pulse(&self, pwd: f32) -> Self;
}

impl OscillatorMath for f32 {
    fn tri(&self) -> Self {
        // Slope result from the calculation of the linear equation for the triangle function
        let value = *self % (2.0 * PI);
        let m = 2.0 / PI;
        let y0 = -1f32;
        let y1 = 3f32;

        if value < PI {
            linear(value, y0, m)
        } else {
            linear(value, y1, -m)
        }
    }

    fn saw(&self) -> Self {
        let value = *self % (2.0 * PI);
        let m = -1.0 / PI;
        let y = 1.0;

        linear(value, y, m)
    }

    fn sqr(&self) -> Self {
        if *self % (2.0 * PI) > PI {
            1.0
        } else {
            -1.0
        }
    }

    /// pwd ranges from 0 to 2π
    fn pulse(&self, pwd: f32) -> Self {
        let pwd = pwd % (2.0 * PI);
        if *self % (2.0 * PI) > pwd {
            1.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sqr() {
        let test_value_low = 0.0f32;
        let test_value_mid = 0.0f32;
        let test_value_top = 0.0f32;

        assert_eq!(test_value_low.sqr(), 0.0);
        assert_eq!(test_value_mid.sqr(), 0.0);
    }
}
