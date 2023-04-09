mod back_end;
mod bundled_modules;
mod module;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, SampleRate, StreamConfig};
use std::fs;
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

// DEBUGGING, LOGGING
use simplelog::__private::paris::Logger;
use simplelog::*;

// MY STUFF
use crate::module::AuxInputBuilder;
use back_end::{get_preferred_config, Channels};
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
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("Failed to start simplelog");
    let mut logger = Logger::new();

    // FILL BUFFER
    info!("<b>Running <blue>demo program</>");
    let signal_duration: i32 = 1000; // milliseconds
    let mut stream_buffer = module_chain(signal_duration * SAMPLE_RATE / 1000);

    output_wav(stream_buffer.clone(), "test.wav");
    module_chain_from_yaml(); // TODO TESTING

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

    /* TODO: real time processing
    let sample_rate = config.sample_rate.0 as f32;
    // A closure to generate a sin wave
    let mut sample_clock = 0f32;
    let frequency = 440.0;


    let mut next_value = move || {
        // CALL FIRST MODULE
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * PI / sample_rate).sin()
    };
     */

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

fn module_chain_from_yaml() -> Vec<f32> {
    use yaml_rust::{YamlEmitter, YamlLoader};
    println!(); // Logger cleanspace

    let yaml = &fs::read_to_string("layouts/layout.yaml").unwrap();

    let doc = YamlLoader::load_from_str(yaml).unwrap();
    let doc = &doc[0];

    let version = *&doc["version"].as_f64().unwrap_or(0.0);
    let layout = &doc["layout"];

    if version != 0.2f64 {
        error!("<b>Please use the <red>latest YAML</> <b>version.</>");
        panic!("")
    } else {
        info!("<b>Using <blue>version</> <b>{}</>", version);
    }

    for module in layout.clone().into_iter() {
        let module = &module["module"];
        let module_type = &module["type"];

        if module_type.as_str().is_none() {
            error!("<b>Module <red>type</> <b>not specified.</>");
            panic!();
        }

        let module_type = module_type.as_str().unwrap();
        let config = &module["config"];
        let name = config["name"].as_str();

        match module_type {
            "oscillator" => {
                let sample_rate = config["sample_rate"].as_i64();
                let amp = config["amplitude"].as_f64();
                let freq = config["frequency"].as_f64();
                let phase = config["phase"].as_f64();

                let osc = OscillatorBuilder::new()
                    .with_all_yaml_fmt(name, sample_rate, amp, freq, phase)
                    .build()
                    .unwrap();
            }

            _ => {
                error!("<b>Module type <red>not found</><b>.</>");
            }
        }

        #[cfg(feature = "verbose_modules")]
        {
            info!("<b>MODULE {}</>", module["id"].as_i64().unwrap());
            info!("  |_ type: {}</>", module_type);
            let out_to = module["output-to"].as_i64().unwrap();
            info!("  |_ output to module: {}</>", out_to);
            info!("  |_ config");
            info!("     |_ name: {}", name.unwrap_or("not defined"));
            // if let Some(sample_rate) = sample_rate {
            //     info!("     |_ sample rate: {}", sample_rate);
            // }
            // if let Some(amp) = amp {
            //     info!("     |_ amplitude: {}", amp);
            // }
            // if let Some(freq) = freq {
            //     info!("     |_ frequency: {}", freq);
            // }
            // if let Some(phase) = phase {
            //     info!("     |_ phase: {}", phase);
            // }

            println!();
        }
    }
    exit(0);

    Vec::with_capacity(44100)
}

fn module_chain(buffer_length: i32) -> Vec<f32> {
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

    modulator.fill_buffer(&mut modulator_buffer);

    let mut aux = AuxInputBuilder::new("frequency", modulator_buffer)
        .with_max(20.0)
        .with_min(10.0)
        .build()
        .unwrap();

    // carrier.fill_buffer_w_aux(&mut buffer, Some(vec![&mut aux]));
    carrier.fill_buffer_w_aux(&mut buffer, Some(vec![&mut aux]));

    #[cfg(feature = "verbose_modules")]
    {
        let mut module = PassTrough::new();
        module.fill_buffer(&mut buffer);
    }
    #[cfg(feature = "module_values")]
    {
        println!();
        info!("<b>You have activated <magenta>Module Values</><b> feature.</>");
        info!("<b>This feature will output the <blue>final value</><b> of the module on each iteration and the values of the <blue>auxiliary inputs</><b>.</>");
        warn!("<b>This is a very <yellow><u>slow</><b> feature. I do only recommend using it with <blue><b>small buffers</>.");
        println!();
    }
    #[cfg(feature = "verbose_modules")]
    {
        info!("<b>You have activated <magenta>Verbose Modules</><b> feature.</>");
        info!("<b>This will output more information about the <blue>inner process</> <b>of the modules.</>");
        println!();
    }
    buffer
}

fn output_wav(buffer: Vec<f32>, filename: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
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
