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

    /// Π_
    /// Pulse 1 [0, π)
    /// Rest -1 [π, 2π)
    fn sqr(&self) -> Self {
        if *self % (2.0 * PI) < PI {
            1.0
        } else {
            -1.0
        }
    }

    /// pwd ranges from 0 to 2π
    /// pulse 1 [0, pulse_width)
    /// rest -1 [pulse_width, 2π)
    fn pulse(&self, pwd: f32) -> Self {
        let pwd = pwd % (2.0 * PI);
        if *self % (2.0 * PI) < pwd {
            1.0
        } else {
            -1.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sqr() {
        let test_value_low = 0.0f32;
        let test_value_mid_a = PI - 0.001;
        let test_value_mid_b = PI;
        let test_value_top_a = 2.0 * PI - 0.001;
        let test_value_top_b = 2.0 * PI;

        assert_eq!(test_value_low.sqr(), 1.0);
        assert_eq!(test_value_mid_a.sqr(), 1.0);
        assert_eq!(test_value_mid_b.sqr(), -1.0);
        assert_eq!(test_value_top_a.sqr(), -1.0);
        assert_eq!(test_value_top_b.sqr(), 1.0);
    }

    #[test]
    fn test_pulse() {
        let test_value_low = 0.0f32;
        let test_value_mid = PI;
        let test_value_top = 2.0 * PI;

        let mut pulse_width = 0.0;
        assert_eq!(test_value_low.pulse(pulse_width), -1.0);
        assert_eq!(test_value_mid.pulse(pulse_width), -1.0);
        assert_eq!(test_value_top.pulse(pulse_width), -1.0);

        let mut pulse_width = PI;
        assert_eq!(test_value_low.pulse(pulse_width), 1.0);
        assert_eq!((PI - 0.001).pulse(pulse_width), 1.0);
        assert_eq!(test_value_mid.pulse(pulse_width), -1.0);
        assert_eq!((PI * 2.0 - 0.001).pulse(pulse_width), -1.0);
        assert_eq!(test_value_top.pulse(pulse_width), 1.0);

        let mut pulse_width = 2.0 * PI - 0.001;
        assert_eq!(test_value_low.pulse(pulse_width), 1.0);
        assert_eq!(test_value_mid.pulse(pulse_width), 1.0);
        assert_eq!(test_value_top.pulse(pulse_width), 1.0);

        let mut pulse_width = 2.0 * PI;
        assert_eq!(test_value_low.pulse(pulse_width), -1.0);
        assert_eq!(test_value_mid.pulse(pulse_width), -1.0);
        assert_eq!(test_value_top.pulse(pulse_width), -1.0);
    }

    #[test]
    fn test_saw() {
        let test_value_low = 0.0f32;
        let test_value_mid = PI;
        let test_value_top_a = 2.0 * PI - 0.001;
        let test_value_top_b = 2.0 * PI;

        assert_eq!(test_value_low.saw(), 1.0);
        assert_eq!(test_value_mid.saw(), 0.0);
        assert_eq!(test_value_top_a.saw(), -0.9996817);
        assert_eq!(test_value_top_b.saw(), 1.0);
    }

    #[test]
    fn test_tri() {
        let test_value_low = 0.0f32;
        let test_value_mid = PI;
        let test_value_top_a = 2.0 * PI - 0.001;
        let test_value_top_b = 2.0 * PI;

        assert_eq!(test_value_low.tri(), -1.0);
        assert_eq!(test_value_mid.tri(), 1.0);
        assert_eq!(test_value_top_a.tri(), -0.9993634);
        assert_eq!(test_value_top_b.tri(), -1.0);
    }
}
