mod back_end;
mod bundled_modules;
mod layout_yaml;
mod module;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, SampleRate, StreamConfig};

use std::thread::sleep;
use std::time::Duration;

// DEBUGGING, LOGGING
use simplelog::__private::paris::Logger;
use simplelog::*;

// MY STUFF
use crate::layout_yaml::buffer_from_yaml;
use crate::module::AuxInputBuilder;
use back_end::{get_preferred_config, output_wav, Channels};
use bundled_modules::debug::{OscDebug, PassTrough};
use bundled_modules::OscillatorBuilder;
use module::Module;

const SAMPLE_RATE: i32 = 44100;

#[allow(dead_code)]
#[cfg(debug_assertions)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() -> Result<(), anyhow::Error> {
    // LOGGER INIT
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to start simplelog");
    // let logger = Logger::new();

    // FILL BUFFER
    info!("<b>Running <blue>demo program</>");
    let signal_duration: i32 = 1000; // milliseconds
    let buffer_size: usize = (signal_duration * SAMPLE_RATE / 1000) as usize;
    // let mut stream_buffer = module_chain(buffer_size);
    test();

    let stream_buffer = buffer_from_yaml("test.yaml", buffer_size);
    output_wav(stream_buffer.clone(), "test.wav");
    // play_buffer(stream_buffer, signal_duration).expect("Playback unsuccessful.");
    play_stream(signal_duration).expect("Playback unsuccessful,");

    info!("<green><tick></> <b>Program finished <green>successfully</><b>.</>");
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

fn play_stream(signal_duration: i32) -> Result<(), anyhow::Error> {
    use module::{GeneratorModuleWrapper, LinkerModuleWrapper};
    use ringbuf::HeapRb;

    let mut logger = Logger::new();
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
        Some(SampleRate(SAMPLE_RATE as u32)),
        Some(Channels::Stereo),
    );

    // open stream
    let config: StreamConfig = supported_config.into();

    let channels = config.channels as usize;

    let osc = OscillatorBuilder::new()
        .with_name("Test")
        .build()
        .expect("Invalid arguments for oscillator");
    let pt = PassTrough::new();

    let rb: HeapRb<f32> = HeapRb::new(10);
    let (producer, consumer) = rb.split();
    let final_rb: HeapRb<f32> = HeapRb::new(10);
    let (prod2, mut cons2) = final_rb.split();

    let mut osc = GeneratorModuleWrapper::new(Box::new(osc), producer);

    let mut pt = LinkerModuleWrapper::new(Box::new(pt), consumer, prod2);
    let mut next_value = move || {
        // CALL FIRST MODULE
        cons2.pop().unwrap_or(0.0)
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

    info!("<b>Signal duration: <u>{} milliseconds</>", signal_duration);
    warn!("<yellow><warn></> <b>The end of the buffer may be filled with <blue>silence</><b>.</>");
    logger.loading("<blue><info></><b> Playing sound</>");
    stream.play()?;

    // duration of the tone
    sleep(Duration::from_millis(signal_duration as u64));

    logger.done();

    Ok(())
}

fn play_buffer(mut stream_buffer: Vec<f32>, signal_duration: i32) -> Result<(), anyhow::Error> {
    let mut logger = Logger::new();
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
        Some(SampleRate(SAMPLE_RATE as u32)),
        Some(Channels::Stereo),
    );

    // open stream
    let config: StreamConfig = supported_config.into();

    let channels = config.channels as usize;

    // If there is no more values in the buffer, silence
    let mut next_value = move || stream_buffer.pop().unwrap_or(0.0);

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

    info!("<b>Signal duration: <u>{} milliseconds</>", signal_duration);
    warn!("<yellow><warn></> <b>The end of the buffer may be filled with <blue>silence</><b>.</>");
    logger.loading("<blue><info></><b> Playing sound</>");
    stream.play()?;

    // duration of the tone
    sleep(Duration::from_millis(signal_duration as u64));

    logger.done();

    Ok(())
}

fn module_chain(buffer_length: usize) -> Vec<f32> {
    // Buffer initialization (1 sec = 44100 samples)
    // let buffer_length = 20;
    let mut buffer: Vec<f32> = vec![0.0; buffer_length as usize];
    let mut modulator_buffer: Vec<f32> = vec![0.0; buffer_length as usize];
    // let mut buffer: Vec<f32> = vec![0.0; 20]; // small BUFFER

    let mut carrier = OscillatorBuilder::new()
        .with_name("Carrier")
        .build()
        .unwrap();
    let mut modulator = OscillatorBuilder::new()
        .with_frequency(440.0)
        .with_name("Modulator")
        .build()
        .unwrap();

    carrier.set_amplitude(1.0);
    carrier.set_frequency(220.0);
    carrier.set_phase(1.0);

    modulator.fill_buffer(&mut modulator_buffer, vec![]);

    let aux = AuxInputBuilder::new("frequency", modulator_buffer)
        .with_max(20.0)
        .with_min(10.0)
        .build()
        .unwrap();

    // carrier.fill_buffer_w_aux(&mut buffer, Some(vec![&mut aux]));
    carrier.fill_buffer(&mut buffer, vec![aux]);

    #[cfg(feature = "verbose_modules")]
    {
        let mut module = PassTrough::new();
        module.fill_buffer(&mut buffer, vec![]);
    }

    #[cfg(feature = "verbose_modules")]
    {
        info!("<b>You have activated <magenta>Verbose Modules</><b> feature.</>");
        info!("<b>This will output more information about the <blue>inner process</> <b>of the modules.</>");
        println!();
    }
    buffer
}

fn test() {
    use module::{GeneratorModuleWrapper, LinkerModuleWrapper};
    use ringbuf::HeapRb;
    use std::process::exit;

    let osc = OscillatorBuilder::new()
        .with_name("Test")
        .build()
        .expect("Invalid arguments for oscillator");
    let pt = PassTrough::new();

    let rb: HeapRb<f32> = HeapRb::new(10);
    let (producer, consumer) = rb.split();
    let final_rb: HeapRb<f32> = HeapRb::new(10);
    let (prod2, mut cons2) = final_rb.split();

    let mut osc = GeneratorModuleWrapper::new(Box::new(osc), producer);

    let mut pt = LinkerModuleWrapper::new(Box::new(pt), consumer, prod2);

    for i in 0..10 {
        let time = i as f32;
        osc.gen_sample(time);
        pt.gen_sample(time);
    }

    while !cons2.is_empty() {
        info!("{}", cons2.pop().unwrap());
    }
}
