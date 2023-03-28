use crate::module::{Module, Parameter, ParameterFactory};
use crate::SAMPLE_RATE;
use std::f32::consts::PI;

/// The oscillator is the genesis of the chain. It does generate a raw signal
/// following certain properties defined by its attributes.
///
/// # Usage
/// To generate a **new oscillator**, use the [OscillatorFactory] instead.
///
/// To **change the behaviour** of the instance, use the functions named after the parameters
/// (right below).
///
/// # Parameters
/// * **Amplitude (A)**: translates to volume (gain). Ranges from 0 to 1. Taking it further will
/// cause the output to clip.  
/// * **Frequency (f)**: translates to tone (musical note). Ranges all through the human audible
/// range; from 10 Hz to 22kHz.
/// * **Phase (Φ)**: sets the initial position of the wave and, thus, a delay for the rest of
/// values over time. Represented in radians, it ranges from 0 to 2π. If the value was
/// set to π, the wave would start from the middle and offset every value after.
pub struct Oscillator {
    /// Inner workings for time tracing
    clock: f32,
    /// Amount of samples in a second
    sample_rate: f32,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    /// Parameter list
    parameters: Vec<Parameter>,
}

impl Module for Oscillator {
    fn behaviour(&self, _in_data: f32) -> f32 {
        (self.clock * self.frequency * 2.0 * PI * self.amplitude + self.phase / self.sample_rate)
            .sin()
    }

    fn get_parameter_list_mutable(&mut self) -> &mut Vec<Parameter> {
        &mut self.parameters
    }

    fn get_parameter_list(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn tick(&mut self) {
        self.clock = (self.clock + 1.0) % self.sample_rate;
    }

    fn get_clock(&self) -> f32 {
        self.clock
    }
}

/// Some shortcut methods for the parameters.
impl Oscillator {
    /// Shortcut method for the amplitude parameter
    pub fn set_amplitude(&mut self, amp: f32) {
        self.get_parameter_mutable("amplitude").unwrap().set(amp);
    }

    /// Shortcut method for the frequency parameter
    pub fn set_frequency(&mut self, freq: f32) {
        self.get_parameter_mutable("frequency").unwrap().set(freq);
    }

    /// Shortcut method for the phase parameter
    pub fn set_phase(&mut self, phase: f32) {
        self.get_parameter_mutable("phase").unwrap().set(phase);
    }

    pub fn get_amplitude(&self) -> f32 {
        self.get_parameter("amplitude").unwrap().get_value()
    }

    pub fn get_frequency(&self) -> f32 {
        self.get_parameter("frequency").unwrap().get_value()
    }

    pub fn get_phase(&self) -> f32 {
        self.get_parameter("phase").unwrap().get_value()
    }
}

pub struct OscillatorFactory {
    sample_rate: Option<f32>,
    frequency: Option<f32>,
    amplitude: Option<f32>,
    phase: Option<f32>,
    parameters: Option<Vec<Parameter>>,
}

impl OscillatorFactory {
    pub fn new() -> Self {
        Self {
            sample_rate: None,
            frequency: None,
            amplitude: None,
            phase: None,
            parameters: None,
        }
    }

    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = Some(sample_rate as f32);
        self
    }

    pub fn with_frequency(mut self, freq: f32) -> Self {
        self.frequency = Some(freq);
        self
    }

    pub fn with_amplitude(mut self, amp: f32) -> Self {
        self.amplitude = Some(amp);
        self
    }

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = Some(phase);
        self
    }

    pub fn build(self) -> Result<Oscillator, String> {
        let sample_rate = self.sample_rate.unwrap_or(SAMPLE_RATE as f32);
        let frequency = self.frequency.unwrap_or(440.0);
        let amplitude = self.amplitude.unwrap_or(1.0);
        let phase = self.phase.unwrap_or(0.0);

        Ok(Oscillator {
            clock: 0.0,
            sample_rate,
            frequency,
            amplitude,
            phase,
            parameters: vec![
                ParameterFactory::new("amplitude".to_string())
                    .with_default(0.8)
                    .build()
                    .unwrap(),
                ParameterFactory::new("frequency".to_string())
                    .with_max(22000.0)
                    .with_min(10.0)
                    .with_step(1.0)
                    .with_default(440.0)
                    .build()
                    .unwrap(),
                ParameterFactory::new("phase".to_string())
                    .with_max(PI * 2.0)
                    .build()
                    .unwrap(),
            ],
        })
    }
}

#[cfg(test)]
mod oscillator_factory_tests {
    use crate::bundled_modules::OscillatorFactory;
    use crate::module::Module;
    use simplelog::__private::paris::Logger;
    use std::f32::consts::PI;

    fn get_logger() -> Logger<'static> {
        Logger::new()
    }

    #[test]
    fn test_get_amplitude() {
        let mut osc = OscillatorFactory::new().build().unwrap();

        osc.set_amplitude(1.0);
        let value = (&osc).get_amplitude();
        assert_eq!(1.0, value);

        osc.set_amplitude(0.0);
        let value = (&osc).get_amplitude();
        assert_eq!(0.0, value);
    }

    #[test]
    fn test_get_set_frequency() {
        let mut osc = OscillatorFactory::new().build().unwrap();

        osc.set_frequency(220.0);
        let value = (&osc).get_frequency();
        assert_eq!(220.0, value);

        osc.set_frequency(660.0);
        let value = (&osc).get_frequency();
        assert_eq!(660.0, value);
    }

    #[test]
    fn test_get_set_phase() {
        let mut osc = OscillatorFactory::new().build().unwrap();

        osc.set_phase(PI / 3.0);
        let value = (&osc).get_phase();
        assert_eq!(PI / 3.0, value);

        osc.set_phase(PI);
        let value = (&osc).get_phase();
        assert_eq!(PI, value);
    }

    #[test]
    fn test_empty() {
        let mut logger = get_logger();
        logger.info("<b>Running test for oscillator factory with no arguments</>");

        let osc = OscillatorFactory::new().build().unwrap();

        assert_eq!(osc.sample_rate, 44100.0, "Sample mismatch");
        assert_eq!(osc.amplitude, 1.0, "Amplitude mismatch");
        assert_eq!(osc.frequency, 440.0, "Frequency mismatch");
        assert_eq!(osc.phase, 0.0, "Phase mismatch");
        assert_eq!(osc.clock, 0.0, "Clock mismatch");

        let amp = (&osc).get_parameter("amplitude");
        let phase = (&osc).get_parameter("phase");
        let freq = (&osc).get_parameter("frequency");

        assert!(amp.is_some(), "Default amplitude parameter missing");
        assert!(phase.is_some(), "Default phase parameter missing");
        assert!(freq.is_some(), "Default frequency parameter missing");
    }
}
