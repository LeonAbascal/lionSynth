mod back_end;
mod bundled_modules;
mod layout_yaml;
mod module;

// LOGGING
use simplelog::*;

// MY STUFF
use back_end::output_wav;
use back_end::play_buffer;
use layout_yaml::{buffer_from_yaml, play_from_yaml};

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
    show_features_info();
    let signal_duration: i32 = 1000; // milliseconds
    let buffer_size: usize = (signal_duration * SAMPLE_RATE / 1000) as usize;

    let stream_buffer = buffer_from_yaml("test.yaml", buffer_size);
    output_wav(stream_buffer.clone(), "test.wav", SAMPLE_RATE);

    play_buffer(stream_buffer, signal_duration, SAMPLE_RATE).expect("Error during playback.");
    play_from_yaml("test.yaml", signal_duration, SAMPLE_RATE).expect("Error during playback.");
    info!("<green><tick></> <b>Program finished <green>successfully</><b>.</>");
    Ok(())
}

fn show_features_info() {
    #[cfg(feature = "verbose_modules")]
    {
        info!("<b>You have activated <magenta>Verbose Modules</><b> feature.</>");
        info!("<b>This will output more information about the <blue>inner process</> <b>of the modules.</>");
        println!();
    }
}
