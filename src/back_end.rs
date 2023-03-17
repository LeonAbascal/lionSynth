// This files contains some custom stuff for initializing the back-end

use cpal::traits::{DeviceTrait};
use cpal::{Device, SupportedStreamConfig, SampleRate, SampleFormat, SupportedStreamConfigRange};
#[cfg(debug_assertions)]
use cpal::{SupportedOutputConfigs};

/// Looks up for a supported config with a specific sample format.
///
/// # Arguments
/// * `device` - a `Device` from which to get the **supported configurations**.
/// * `sample_format` - (optional) a `SampleFormat` with the **preferred format** for each **sample**.
/// * `sample_rate` - (optional) a `SampleRate`. If not set it will default to the max (not recommended).
/// * `channel_amt` - (optional) the maximum amount of channels to use. Mono or Stereo is recommended.
///
/// # Return
/// Returns the first `SupportedStreamConfig` fulfilling the requirements from the arguments.

pub fn get_preferred_config(
    device: &Device,
    sample_format: Option<SampleFormat>,
    sample_rate: Option<SampleRate>,
    channel_amt: Option<Channels>,
) -> SupportedStreamConfig {
    let config = query_config(device, channel_amt, sample_format, sample_rate);

    if cfg!(debug_assertions) {
        println!(
            "PREFERRED CONFIG for {}",
            device.name().expect("Couldn't read device name")
        );
        println!(" |_ channels: {}", config.channels());
        println!(" |_ sample_rate: {}", config.sample_rate().0);
        println!(" |_ buffer size: {:?}", config.buffer_size());
        println!(" |_ sample format: {:?}", config.sample_format());
        println!();
    }

    config
}

#[cfg(debug_assertions)]
#[allow(unused)]
fn print_all_configs(device: &Device) {
    let supported_configs_range: SupportedOutputConfigs = device
        .supported_output_configs()
        .expect("error while querying configs");

    if cfg!(debug_assertions) {
        for (ix, config) in supported_configs_range.enumerate() {
            println!("{}: {:?}", ix, config);
        }
        println!();
    }
}

pub fn query_configurations(
    device: &Device,
    channel_amt: Option<Channels>,
    sample_format: Option<SampleFormat>,
) -> Vec<SupportedStreamConfigRange> {
    if cfg!(debug_assertions) {
        println!("QUERYING {:?} device UNDER", device.name().unwrap());
        println!("  |_ channel amount: {:?}", channel_amt);
        println!("  |_ sample format: {:?}", sample_format);
        println!();
    }
    let supported_configs = device
        .supported_output_configs()
        .expect("error while querying configs")
        // Check the sample format
        .filter(|config| match &sample_format {
            None => true,
            Some(a) => config.sample_format() == (*a),
        })
        // Check the channel amount
        .filter(|config| match &channel_amt {
            None => true,
            Some(a) => (*a).get_amt() >= config.channels() as u8,
        })
        // to vector
        .collect::<Vec<SupportedStreamConfigRange>>();

    // RESULT PRINTS
    if cfg!(debug_assertions) {
        println!("CONFIGURATION MATCHES");
        let configs = supported_configs.clone();
        for item in configs {
            println!("  |_ {:?}", item);
        }
        println!();
    }

    supported_configs
}

pub fn query_config(
    device: &Device,
    channel_amt: Option<Channels>,
    sample_format: Option<SampleFormat>,
    sample_rate: Option<SampleRate>,
) -> SupportedStreamConfig {
    let mut supported_configs = query_configurations(device, channel_amt, sample_format);

    let range = supported_configs
        .pop()
        .expect("No possible configuration could be found. Try widening the search.");

    match sample_rate {
        None => range.with_max_sample_rate(),
        Some(x) => range.with_sample_rate(x),
    }
}

/// An enumeration for specifying an amount of channels and easily differentiate the most common cases (mono and stereo).
#[derive(Debug)]
#[allow(dead_code)]
pub enum Channels {
    /// A single channel
    Mono,
    /// Two channels
    Stereo,
    /// Any given amount of channels
    Multi(u8),
}

impl Channels {
    /// Translates the `enum` to a value for ease.
    /// # Example
    /// ```
    /// let x: u8 = Channels::Stereo.get_amt(); // returns 2
    /// ```
    pub fn get_amt(&self) -> u8 {
        match *self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Multi(x) => x,
        }
    }
}
