mod generic_modules;
mod module;
mod oscillator_math;
mod back_end;

use std::f32::consts::PI;
use cpal::{Device, FromSample, Sample, SampleFormat, SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use back_end::{Channels, get_preferred_config};
use module::Module;
use generic_modules::{OscDebug, PassTrough};

#[allow(dead_code)]
#[cfg(debug_assertions)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() -> Result<(), anyhow::Error> {

    // LAUNCH LITTLE TEST
    test();

    // get default host
    let host = cpal::default_host();

    // get default device
    let device: Device = host
        .default_output_device()
        .expect("no default output device available. Please check if one is selected");

    // load config
    let supported_config = get_preferred_config(
        &device,
        Some(SampleFormat::F32),
        Some(SampleRate(44100)),
        Some(Channels::Stereo),
    );

    // open stream
    let config: StreamConfig = supported_config.into();

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // A closure to generate a sin wave
    let mut sample_clock = 0f32;
    let frequency = 440.0;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    // call a function to let cpal output the stream
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    // duration of the tone
    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

// from cpal examples beep.rs
fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
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

#[allow(dead_code)]
fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

fn test() {
    let mut buffer: Vec<f32> = Vec::with_capacity(20); // 20 samples
    buffer.resize(20, 0.0);

    println!("\nDEBUG OSC--");
    let mut module2 = OscDebug::new(44100);
    module2.fill_buffer(&mut buffer);

    println!("\nPASS THROUGH--");
    let mut module = PassTrough::new();
    module.fill_buffer(&mut buffer);

    println!();
}
