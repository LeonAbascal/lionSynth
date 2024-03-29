use crate::bundled_modules::osc::oscillator_math::{OscillatorMath, WaveShape};
use crate::module::{Module, Parameter, ParameterBuilder};
use crate::SAMPLE_RATE;
use simplelog::{error, info};
use std::f32::consts::PI;

// TODO: add wave shape to doc
/// The oscillator is the genesis of the chain. It does generate a raw signal
/// following certain properties defined by its attributes.
///
/// # Usage
/// To generate a **new oscillator**, use the [OscillatorBuilder] instead.
///
/// To **change the behaviour** of an instance, use the functions named after the parameters.
/// * [set_amplitude](fn@Oscillator::set_amplitude)
/// * [set_frequency](fn@Oscillator::set_frequency)
/// * [set_phase](fn@Oscillator::set_phase)
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
/// For more detailed information on default values check [OscillatorBuilder].
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
/// `t` the time given by a coordinator entity.
pub struct Oscillator {
    /// The maximum amplitude of the wave. Translates to volume (gain). A value greater than one will result in clipping.
    amplitude: Parameter,
    /// The frequency of the wave. Translates to tone.
    frequency: Parameter,
    /// The phase of the wave, ie, the point at which the cycle starts.
    phase: Parameter,
    /// The shape of the wave, which will produce a different timbre.
    wave_shape: WaveShape,
    /// The width of the pulse. Only works with a pulse wave (this is not PWM).
    pulse_width: Parameter,
    /// Name of the module (debugging)
    name: String,
}

impl Module for Oscillator {
    fn behavior(&self, _in_data: f32, time: f32) -> f32 {
        let mut value = ((time * self.get_frequency() * 2.0 * PI) + self.get_phase());

        value = match self.get_wave() {
            WaveShape::Saw => value.saw(),
            WaveShape::Square => value.sqr(),
            WaveShape::Pulse(x) => value.pulse(*x),
            WaveShape::Sine => value.sin(),
            WaveShape::Triangle => value.tri(),
            _ => {
                error!("<b>Wave shape not supported. Generating a sine wave by default.</>");
                value.sin()
            }
        };

        return value * self.get_amplitude();
    }

    fn get_parameters(&self) -> Option<Vec<&Parameter>> {
        Some(vec![&self.amplitude, &self.frequency, &self.phase])
    }

    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>> {
        Some(vec![
            &mut self.amplitude,
            &mut self.frequency,
            &mut self.phase,
        ])
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
        self.amplitude.set(amp);
    }

    /// Shortcut method for setting the frequency parameter.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency.set(freq);
    }

    /// Shortcut method for setting the phase parameter.
    pub fn set_phase(&mut self, phase: f32) {
        self.phase.set(phase);
    }

    /// Method for setting the shape of the wave.
    pub fn set_wave(&mut self, wave: WaveShape) {
        self.wave_shape = wave;
    }

    /// Shortcut method for getting the amplitude parameter.
    pub fn get_amplitude(&self) -> f32 {
        self.amplitude.get_value()
    }

    /// Shortcut method for getting the frequency parameter.
    pub fn get_frequency(&self) -> f32 {
        self.frequency.get_value()
    }

    /// Shortcut method for getting the phase parameter.
    pub fn get_phase(&self) -> f32 {
        self.phase.get_value()
    }

    /// Methods for getting the wave currently selected.
    pub fn get_wave(&self) -> &WaveShape {
        &self.wave_shape
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
    frequency: Option<f32>,
    amplitude: Option<f32>,
    phase: Option<f32>,
    wave: Option<WaveShape>,
    pulse_width: Option<f32>,
    name: Option<String>,
}

impl OscillatorBuilder {
    /// Sets the defaults for the oscillator (no parameters).
    pub fn new() -> Self {
        Self {
            name: None,
            frequency: None,
            amplitude: None,
            phase: None,
            wave: None,
            pulse_width: None,
        }
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

    pub fn with_wave(mut self, wave: WaveShape) -> Self {
        self.wave = Some(wave);
        self
    }

    pub fn with_pulse_width(mut self, pw: f32) -> Self {
        self.pulse_width = Some(pw);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_all_yaml_fmt(
        name: Option<&str>,
        amplitude: Option<f64>,
        frequency: Option<f64>,
        phase: Option<f64>,
        wave: Option<WaveShape>,
        pw: Option<f64>,
    ) -> Self {
        let name = match name {
            Some(x) => Some(x.to_string()),
            None => None,
        };
        let amplitude = match amplitude {
            Some(x) => Some(x as f32),
            None => None,
        };
        let frequency = match frequency {
            Some(x) => Some(x as f32),
            None => None,
        };
        let phase = match phase {
            Some(x) => Some(x as f32),
            None => None,
        };
        let pulse_width = match pw {
            Some(x) => Some(x as f32),
            None => None,
        };

        Self {
            name,
            amplitude,
            frequency,
            phase,
            wave,
            pulse_width,
        }
    }

    /// Tries to generate an Oscillator from the given configuration.
    ///
    /// # Default values:
    /// * Frequency: 440 Hz
    /// * Amplitude: 1.0
    /// * Phase: 0 radians
    ///
    /// # Expected errors
    /// * Frequency, amplitude or phase out of range.
    pub fn build(self) -> Result<Oscillator, String> {
        let name = match self.name {
            Some(name) => format!("{} Oscillator", name),
            None => format!("Oscillator"),
        };

        let frequency = self.frequency.unwrap_or(440.0);
        let amplitude = self.amplitude.unwrap_or(1.0);
        let phase = self.phase.unwrap_or(0.0);
        let wave = self.wave.unwrap_or_default();
        let pulse_width = self.pulse_width.unwrap_or(PI);

        // Value check left for the Parameter factories

        Ok(Oscillator {
            name,
            amplitude: ParameterBuilder::new("amplitude".to_string())
                .with_default(amplitude)
                .build()
                .expect("Invalid amplitude value"),

            frequency: ParameterBuilder::new("frequency".to_string())
                .with_max(22000.0)
                .with_min(10.0)
                .with_default(frequency)
                .build()
                .expect("Invalid frequency value"),

            phase: ParameterBuilder::new("phase".to_string())
                .with_max(PI * 2.0)
                .with_default(phase)
                .build()
                .expect("Invalid phase value"),
            wave_shape: wave,

            pulse_width: ParameterBuilder::new("pulse width".to_string())
                .with_max(2.0 * PI)
                .with_min(0.0)
                .with_default(pulse_width)
                .build()
                .expect("Invalid pulse width"),
        })
    }
}

#[cfg(test)]
mod oscillator_builder_tests {
    use super::Module;
    use super::OscillatorBuilder;
    use crate::bundled_modules::Oscillator;
    use crate::module::Clock;
    use crate::SAMPLE_RATE;
    use simplelog::__private::paris::Logger;
    use std::f32::consts::PI;

    fn get_logger() -> Logger<'static> {
        Logger::new()
    }

    #[test]
    fn test_default() {
        let mut logger = get_logger();
        let mut clock = Clock::new(SAMPLE_RATE);

        logger.info("<b>Running test for oscillator builder with no arguments</>");

        let osc = OscillatorBuilder::new().build().unwrap();

        assert_eq!(clock.get_time(), 0.0, "Clock mismatch");

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
        let mut clock = Clock::new(SAMPLE_RATE);
        let osc = OscillatorBuilder::new()
            .with_amplitude(0.5)
            .with_frequency(220.0)
            .with_phase(1.0)
            .build()
            .unwrap();

        assert_eq!(clock.get_time(), 0.0, "Clock mismatch");

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

    #[test]
    #[should_panic]
    fn test_invalid_amplitude_max() {
        OscillatorBuilder::new()
            .with_amplitude(1.1)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_amplitude_min() {
        OscillatorBuilder::new()
            .with_amplitude(-0.1)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_frequency_min() {
        OscillatorBuilder::new()
            .with_frequency(-1.0)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_frequency_max() {
        OscillatorBuilder::new()
            .with_frequency(22000.1)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_phase_min() {
        OscillatorBuilder::new().with_phase(-1.0).build().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_phase_max() {
        OscillatorBuilder::new()
            .with_phase(PI * 3.0)
            .build()
            .unwrap();
    }
}

#[cfg(test)]
mod oscillator_tests {
    use super::OscillatorBuilder;
    use std::f32::consts::PI;

    #[test]
    fn test_set_amplitude() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_amplitude(0.5);
        let prev = osc.get_amplitude();
        osc.set_amplitude(1.1);
        let mut post = osc.get_amplitude();
        assert_eq!(prev, post, "Amplitude should not surpass the maximum.");

        osc.set_amplitude(-1.0);
        post = osc.get_amplitude();
        assert_eq!(prev, post, "Amplitude should not surpass the minimum.");
    }

    #[test]
    fn test_set_frequency() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_frequency(440.0);
        let prev = osc.get_frequency();
        osc.set_frequency(22100.0);
        let mut post = osc.get_frequency();
        assert_eq!(prev, post, "Frequency should not surpass the maximum.");

        osc.set_frequency(-1.0);
        post = osc.get_frequency();
        assert_eq!(prev, post, "Frequency should not surpass the minimum.");
    }

    #[test]
    fn test_set_phase() {
        let mut osc = OscillatorBuilder::new().build().unwrap();

        osc.set_phase(0.0);
        let prev = osc.get_phase();
        osc.set_phase(PI * 3.0);
        let mut post = osc.get_phase();
        assert_eq!(prev, post, "Phase should not surpass the maximum.");

        osc.set_phase(-1.0);
        post = osc.get_phase();
        assert_eq!(prev, post, "Phase should not surpass the minimum.");
    }

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
