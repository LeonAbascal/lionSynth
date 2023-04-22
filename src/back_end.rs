// This files contains some custom stuff for initializing the back-end

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(debug_assertions)]
use cpal::SupportedOutputConfigs;
use cpal::{
    Device, FromSample, Sample, SampleFormat, SampleRate, StreamConfig, SupportedStreamConfig,
    SupportedStreamConfigRange,
};
use simplelog::__private::paris::Logger;
use simplelog::{info, warn};
use std::fs;
use std::thread::sleep;
use std::time::Duration;

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
        info!(
            "<b>PREFERRED CONFIG for <red>{}</>",
            device.name().expect("Couldn't read device name")
        );
        info!(" |_ channels: {}", config.channels());
        info!(" |_ sample_rate: {}", config.sample_rate().0);
        info!(" |_ buffer size: {:?}", config.buffer_size());
        info!(" |_ sample format: {:?}\n", config.sample_format());
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

/// Query every configuration meeting certain conditions.
/// # Arguments
/// * `device` - a `cpal::Device` from which get the configuration
/// * `channel_amt` - amount of channels we want available. Will get from the amount onwards.
/// * `sample_format` - the format in which data is going to be handled (cpal::SampleFormat)
///
/// # Returns
/// A vector containing every cpal::SupportedStreamConfigRange matching the requirements
pub fn query_configurations(
    device: &Device,
    channel_amt: Option<Channels>,
    sample_format: Option<SampleFormat>,
) -> Vec<SupportedStreamConfigRange> {
    if cfg!(debug_assertions) {
        info!(
            "<b>QUERYING <red>{:?} device</><b> UNDER</>",
            device.name().unwrap()
        );
        info!("  |_ channel amount: {:?}", channel_amt);
        info!("  |_ sample format: {:?}\n", sample_format);
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
        info!("<b>CONFIGURATION MATCH LIST</>");
        let configs = supported_configs.clone();
        for item in configs {
            info!("  |_ {:?}", item);
        }
        println!();
    }

    supported_configs
}

/// Queries the first configuration found meeting certain conditions.
/// # Arguments
/// * `device` - a `cpal::Device` from which get the configuration
/// * `channel_amt` - amount of channels we want available. Will default to the lowest possible one
/// * `sample_format` - the format in which data is going to be handled (cpal::SampleFormat)
///
/// # Returns
/// A cpal::SupportedStreamConfigRange matching the requirements.
pub fn query_config(
    device: &Device,
    channel_amt: Option<Channels>,
    sample_format: Option<SampleFormat>,
    sample_rate: Option<SampleRate>,
) -> SupportedStreamConfig {
    println!();
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

pub fn output_wav(buffer: Vec<f32>, filename: &str, sample_rate: i32) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    info!("<b>Running <magenta>hound</> <b>to generate a wav file.</>");
    info!("  <b>|_ Channels: <cyan>{}</>", spec.channels);
    info!("  <b>|_ Bits per sample: <cyan>{}</>", spec.bits_per_sample);
    info!(
        "  <b>|_ Sample format: {}</>",
        match spec.sample_format {
            hound::SampleFormat::Int => "<yellow>int",
            hound::SampleFormat::Float => "<cyan>float",
        }
    );

    let subdir = "exports".to_string();
    info!("  <b>|_ Export directory: <green>.{}/</>", subdir);
    info!("  <b>|_ File name: <green>{}</>", filename);

    fs::create_dir_all(&subdir).unwrap();
    let filename = subdir + "/" + filename;

    let mut test_writer = hound::WavWriter::create(filename, spec).unwrap();
    let amplitude = i16::MAX as f32;
    for sample in buffer {
        test_writer
            .write_sample((amplitude * sample) as i16)
            .unwrap();
    }

    test_writer.finalize().unwrap();
}

pub fn play_buffer(
    mut buffer: Vec<f32>,
    signal_duration: i32,
    sample_rate: i32,
) -> Result<(), anyhow::Error> {
    let mut logger = Logger::new();

    let host = cpal::default_host();

    // get default device
    let device: Device = host
        .default_output_device()
        .expect("no default output device available. Please check if one is selected");

    // load config
    let supported_config = get_preferred_config(
        &device,
        Some(SampleFormat::F32),
        Some(SampleRate(sample_rate as u32)),
        Some(Channels::Stereo),
    );

    // open stream
    let config: StreamConfig = supported_config.into();

    let channels = config.channels as usize;

    // If there is no more values in the buffer, silence
    let mut next_value = move || buffer.pop().unwrap_or(0.0);
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;

    info!("<b>Signal duration: <u>{} milliseconds</>", signal_duration);
    warn!("<yellow><warn></> <b>The end of the buffer may be filled with <blue>silence</><b>.</>");
    logger.loading("<blue><info></><b> Playing sound</>");
    stream.play()?;

    // duration of the tone
    sleep(Duration::from_millis(signal_duration as u64));

    logger.done();

    Ok(())
}

/// This function fills the data in batches. Is called by the cpal when it considers timely.
pub fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
