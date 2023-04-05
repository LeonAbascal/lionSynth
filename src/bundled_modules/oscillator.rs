use crate::module::{AuxiliaryInput, Module, Parameter, ParameterBuilder};
use crate::SAMPLE_RATE;
use std::f32::consts::PI;
use std::fmt;
use std::fmt::{write, Formatter};

/// The oscillator is the genesis of the chain. It does generate a raw signal
/// following certain properties defined by its attributes.
///
/// # Usage
/// To generate a **new oscillator**, use the [OscillatorBuilder] instead.
///
/// To **change the behaviour** of an instance, use the functions named after the parameters
/// (right below).
///
/// # Parameters
/// The following parameters are available for modifying to the user:
/// * **Amplitude (A)**: translates to volume (gain). Ranges from 0 to 1. Taking it further will
/// cause the output to clip.  
/// * **Frequency (f)**: translates to tone (musical note). Ranges all through the human audible
/// range; from 10 Hz to 22kHz.
/// * **Phase (φ)**: sets the initial position of the wave and, thus, a delay for the rest of
/// values over time. Represented in radians, it ranges from 0 to 2π. If the value was
/// set to π, the wave would start from the middle and offset every value after.
///
/// # Behaviour
/// The generation of a signal follows a simple formula:
///
/// `x = A * sin(f * t + φ)`
///
/// Where `x` is the value at `t` time,
/// `A` is the maximum amplitude of the wave,
/// `φ` the phase and
/// `f` the frequency.
///
/// `t` gets calculated with the clock of the oscillator and the sample rate.
pub struct Oscillator {
    /// Inner workings for time tracing
    clock: f32,
    /// Amount of samples in a second
    sample_rate: f32,
    /// Parameter list
    parameters: Vec<Parameter>,
    /// Name of the module (debugging)
    name: String,
}

impl Module for Oscillator {
    fn behaviour(&self, _in_data: f32) -> f32 {
        ((self.clock * self.get_frequency() * 2.0 * PI / self.sample_rate) + self.get_phase()).sin()
            * self.get_amplitude()
    }

    fn get_parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn get_parameters_mutable(&mut self) -> &mut Vec<Parameter> {
        &mut self.parameters
    }

    fn inc_clock(&mut self) {
        self.clock = (self.clock + 1.0) % self.sample_rate;
    }

    fn get_clock(&self) -> f32 {
        self.clock
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

/// Some shortcut methods for the parameters. Look at the implementation for reference.
/// ```rust
/// pub fn set_amplitude(&mut self, amp: f32) {
///     self.get_parameter_mutable("amplitude").unwrap().set(amp);
/// }
///
/// pub fn get_amplitude(&self) -> f32 {
///     self.get_parameter("amplitude").unwrap().get_value()
/// }
/// ```
/// Although it is optional, the final user does save time coding and gets cleaner code.
impl Oscillator {
    /// Shortcut method for setting the amplitude parameter.
    pub fn set_amplitude(&mut self, amp: f32) {
        self.get_parameter_mutable("amplitude").unwrap().set(amp);
    }

    /// Shortcut method for setting the frequency parameter.
    pub fn set_frequency(&mut self, freq: f32) {
        self.get_parameter_mutable("frequency").unwrap().set(freq);
    }

    /// Shortcut method for setting the phase parameter.
    pub fn set_phase(&mut self, phase: f32) {
        self.get_parameter_mutable("phase").unwrap().set(phase);
    }

    /// Shortcut method for getting the amplitude parameter.
    pub fn get_amplitude(&self) -> f32 {
        self.get_parameter("amplitude").unwrap().get_value()
    }

    /// Shortcut method for getting the frequency parameter.
    pub fn get_frequency(&self) -> f32 {
        self.get_parameter("frequency").unwrap().get_value()
    }

    /// Shortcut method for getting the phase parameter.
    pub fn get_phase(&self) -> f32 {
        self.get_parameter("phase").unwrap().get_value()
    }
}

/// The [OscillatorBuilder] is the proper way of generating an [Oscillator].
/// # Usage
/// ```rust
/// let mut oscillator = OscillatorBuilder::new().build().unwrap(); // Default oscillator
///
/// let osc = OscillatorBuilder::new() // With most values
///     .with_amplitude(0.5)
///     .with_frequency(220.0)
///     .with_phase(1.0)
///     .build()
///     .unwrap();
/// ```
pub struct OscillatorBuilder {
    sample_rate: Option<f32>,
    frequency: Option<f32>,
    amplitude: Option<f32>,
    phase: Option<f32>,
    parameters: Option<Vec<Parameter>>,
    name: Option<String>,
}

impl OscillatorBuilder {
    /// Sets the defaults for the oscillator (no parameters).
    pub fn new() -> Self {
        Self {
            name: None,
            sample_rate: None,
            frequency: None,
            amplitude: None,
            phase: None,
            parameters: None,
        }
    }

    /// Sets the sample rate of the oscillator.
    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = Some(sample_rate as f32);
        self
    }

    /// Sets the **default** frequency of the *amplitude [parameter](struct@Parameter)*.
    pub fn with_amplitude(mut self, amp: f32) -> Self {
        self.amplitude = Some(amp);
        self
    }

    /// Sets the **default** frequency of the *frequency [parameter](struct@Parameter)*.
    pub fn with_frequency(mut self, freq: f32) -> Self {
        self.frequency = Some(freq);
        self
    }

    /// Sets the **default** frequency of the *phase [parameter](struct@Parameter)*.
    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = Some(phase);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Tries to generate an Oscillator from the given configuration.
    ///
    /// # Default values:
    /// * Sample rate: [SAMPLE_RATE](const@SAMPLE_RATE)
    /// * Frequency: 440 Hz
    /// * Amplitude: 1.0
    /// * Phase: 0 radians
    ///
    /// # Expected errors
    /// * Frequency, amplitude or phase out of range.
    pub fn build(self) -> Result<Oscillator, String> {
        let name = format!("{} {}", self.name.unwrap_or("".to_string()), "Oscillator");
        let sample_rate = self.sample_rate.unwrap_or(SAMPLE_RATE as f32);
        let frequency = self.frequency.unwrap_or(440.0);
        let amplitude = self.amplitude.unwrap_or(1.0);
        let phase = self.phase.unwrap_or(0.0);

        // Value check left for the Parameter factories

        Ok(Oscillator {
            name,
            clock: 0.0,
            sample_rate,
            parameters: vec![
                ParameterBuilder::new("amplitude".to_string())
                    .with_default(amplitude)
                    .build()
                    .unwrap(),
                ParameterBuilder::new("frequency".to_string())
                    .with_max(22000.0)
                    .with_min(10.0)
                    .with_step(1.0)
                    .with_default(frequency)
                    .build()
                    .unwrap(),
                ParameterBuilder::new("phase".to_string())
                    .with_max(PI * 2.0)
                    .with_default(phase)
                    .build()
                    .unwrap(),
            ],
        })
    }
}

#[cfg(test)]
mod oscillator_builder_tests {
    use super::Module;
    use super::OscillatorBuilder;
    use simplelog::__private::paris::Logger;
    use std::f32::consts::PI;

    fn get_logger() -> Logger<'static> {
        Logger::new()
    }

    #[test]
    fn test_empty() {
        let mut logger = get_logger();
        logger.info("<b>Running test for oscillator builder with no arguments</>");

        let osc = OscillatorBuilder::new().build().unwrap();

        assert_eq!(osc.sample_rate, 44100.0, "Default sample mismatch");
        assert_eq!(osc.clock, 0.0, "Clock mismatch");

        let amp = (&osc).get_parameter("amplitude");
        let phase = (&osc).get_parameter("phase");
        let freq = (&osc).get_parameter("frequency");

        assert!(amp.is_some(), "Default amplitude parameter missing");
        assert_eq!(amp.unwrap().get_value(), 1.0, "Default amplitude differs");
        assert!(freq.is_some(), "Default frequency parameter missing");
        assert_eq!(
            freq.unwrap().get_value(),
            440.0,
            "Default frequency differs"
        );
        assert!(phase.is_some(), "Default phase parameter missing");
        assert_eq!(phase.unwrap().get_value(), 0.0, "Default phase differs");
    }

    #[test]
    fn test_all_fields() {
        let osc = OscillatorBuilder::new()
            .with_amplitude(0.5)
            .with_frequency(220.0)
            .with_phase(1.0)
            .with_sample_rate(22000)
            .build()
            .unwrap();

        assert_eq!(osc.sample_rate, 22000.0, "Sample mismatch");
        assert_eq!(osc.clock, 0.0, "Clock mismatch");

        let amp = (&osc).get_parameter("amplitude");
        let phase = (&osc).get_parameter("phase");
        let freq = (&osc).get_parameter("frequency");

        assert!(amp.is_some(), "Amplitude parameter missing");
        assert_eq!(amp.unwrap().get_value(), 0.5, "Amplitude parameter differs");
        assert!(freq.is_some(), "Frequency parameter missing");
        assert_eq!(
            freq.unwrap().get_value(),
            220.0,
            "Frequency parameter differs"
        );
        assert!(phase.is_some(), "Phase parameter missing");
        assert_eq!(phase.unwrap().get_value(), 1.0, "Phase parameter differs");
    }

    // TODO: should panic tests?
}

#[cfg(test)]
mod oscillator_tests {
    use super::OscillatorBuilder;
    use std::f32::consts::PI;

    // TODO: test wrong sets

    #[test]
    fn test_get_amplitude() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_amplitude(1.0);
        let value = (&osc).get_amplitude();
        assert_eq!(1.0, value);

        osc.set_amplitude(0.0);
        let value = (&osc).get_amplitude();
        assert_eq!(0.0, value);
    }

    #[test]
    fn test_get_frequency() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_frequency(220.0);
        let value = (&osc).get_frequency();
        assert_eq!(220.0, value);

        osc.set_frequency(660.0);
        let value = (&osc).get_frequency();
        assert_eq!(660.0, value);
    }

    #[test]
    fn test_get_phase() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_phase(PI / 3.0);
        let value = (&osc).get_phase();
        assert_eq!(PI / 3.0, value);

        osc.set_phase(PI);
        let value = (&osc).get_phase();
        assert_eq!(PI, value);
    }
}
