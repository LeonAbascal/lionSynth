use cpal::{Data, Sample, SampleFormat, FromSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() {
    println!("Hello, world!");
    let host = cpal::default_host();

    let device: cpal::Device = host.default_output_device().expect("no output device available");
    println!("Device name: {}", device.name().expect("Couldn't read device name"));

    let mut supported_configs_range: cpal::SupportedOutputConfigs = device.supported_output_configs()
        .expect("error while querying configs");

    let mut supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    while supported_config.sample_format() != SampleFormat::F32 {
        supported_config = supported_configs_range.next()
            .expect("no supported config?!")
            .with_max_sample_rate();

    }

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let sample_format = supported_config.sample_format();
    println!("sample format: {:?}", sample_format);
    let config = supported_config.into();
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(&config, write_silence::<f32>, err_fn, None),
        SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn, None),
        SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn, None),
        sample_format => panic!("Unsupported sample format '{sample_format}'")
    }.unwrap();

    fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
        for sample in data.iter_mut() {
            *sample = Sample::EQUILIBRIUM;
        }
    }

    stream.play().unwrap();

}
