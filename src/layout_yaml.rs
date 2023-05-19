use crate::back_end::{get_preferred_config, write_data, Channels};
use crate::bundled_modules::debug::*;
use crate::bundled_modules::prelude::Sum3InBuilder;
use crate::bundled_modules::WaveShape;
use crate::bundled_modules::*;
use crate::module::{
    AuxDataHolder, AuxInputBuilder, AuxiliaryInput, CoordinatorEntity, GeneratorModuleWrapper,
    LinkerModuleWrapper, Module, ModuleProducer, ModuleWrapper,
};
use crate::SAMPLE_RATE;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SampleRate, StreamConfig};
use ringbuf::HeapRb;
use simplelog::{error, info, warn};
use std::collections::{HashMap, LinkedList};
use std::f32::consts::PI;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use yaml_rust::{Yaml, YamlLoader};

// TODO test size. Different signal durations may be affected playback
const BATCH_SIZE_RT: usize = 1000;
const YAML_VERSION: &str = "0.5";

use crate::layout_yaml::YamlParsingError::UnknownType;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum YamlParsingError {
    #[error("Missing document version number. Latest version: {0}")]
    MissingVersionNumber(String),
    // #[error("<b>YAML <red>version mismatch</><b>, using version <magenta>{0}</><b>. Latest version: <cyan>{1}</>")]
    #[error("YAML version mismatch, using version {using}. Latest version: {latest}")]
    VersionMismatch { using: String, latest: String },
    #[error("Missing module ID.")]
    MissingID,
    #[error("Found a duplicated ID: {0}")]
    DuplicatedID(i64),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("No module has been linked to the Operating System. Specify the tag 'os-out: true' in the last element of the chain.")]
    MissingOpSysOutput,
    #[error("Missing auxiliary 'lined-with'. Needed to know what parameter to update.")]
    MissingAuxTag,
    #[error("Missing auxiliary 'from-id'. Needed to know where the data comes from.")]
    MissingAuxFromId,
    #[error("Format wrong for field '{field_name}'. Supported format: {supported_format}")]
    WrongFormat {
        field_name: String,
        supported_format: String,
    },
    #[error("Invalid value for '{field_name}' in module with id {module_id}")]
    InvalidValue { field_name: String, module_id: i64 },
    #[error("'{0}' type not known.")]
    UnknownType(String),

    // SUM MODULE
    #[error("{0} is not a valid amount of inputs.")]
    InvalidInputAmount(i64),
}

struct ChainCell {
    from_module: Option<i64>,
    module: Box<dyn Module>,
    auxiliaries: Vec<AuxInfo>,
}

struct AuxInfo {
    from_module: i64,
    linked_with: String,
    max: Option<f32>,
    min: Option<f32>,
}

fn load_yaml(
    file: &str,
    first_module_index: &mut i64,
) -> Result<HashMap<i64, ChainCell>, YamlParsingError> {
    use YamlParsingError::*;

    let mut first_module: Option<i64> = None;
    let path = format!("layouts/{}", file);
    info!("<b>Loading data from <red>{}</><b>.</>", path);
    let yaml = &fs::read_to_string(path).unwrap();

    let doc = YamlLoader::load_from_str(yaml).unwrap();
    let doc = &doc[0];

    let version = &doc["version"];
    let version = match version {
        Yaml::Real(_) => version.as_f64().map(|x| x.to_string()),
        Yaml::String(_) => version.as_str().map(|x| x.to_string()),
        Yaml::BadValue => return Err(MissingVersionNumber(YAML_VERSION.to_string())),
        _ => {
            return Err(WrongFormat {
                field_name: String::from("version"),
                supported_format: String::from("f64, str"),
            })
        }
    };

    let version = version.unwrap();
    if version != YAML_VERSION {
        error!("<b>Please use the <red>latest YAML</> <b>version.</>");
        return Err(VersionMismatch {
            using: version.to_string(),
            latest: YAML_VERSION.to_string(),
        });
    } else {
        info!(
            "<b>Using <magenta>YAML parsing</> <b>version: <b><cyan>{}</>",
            version
        );
    }

    info!("<b>Creating module chain.</>");
    let mut module_chain: HashMap<i64, ChainCell> = HashMap::new();

    let layout = &doc["layout"];
    // TODO add error for missing layout
    for module in layout.clone().into_iter() {
        let module = &module["module"];
        let module_type = &module["type"];
        let module_id = &module["id"];

        // ID CHECKS
        let module_id = match module_id {
            Yaml::Integer(x) => *x,
            Yaml::BadValue => {
                error!("<b>Missing module <red>ID</><b>.</>");

                return Err(MissingField(String::from("ID")));
            }
            _ => {
                error!("<b>Module ID could not be <red>parsed</><b>.</>");
                return Err(WrongFormat {
                    field_name: String::from("id"),
                    supported_format: String::from("i64"),
                });
            }
        };

        info!("> Processing <cyan>module {}</>", module_id);

        if module_chain.contains_key(&module_id) {
            error!("<b>Found a <red>duplicated ID</> <b>value.</>");
            return Err(DuplicatedID(module_id));
        }

        // FIRST BOOL
        if let Some(value) = module["os-out"].as_bool() {
            if value {
                if first_module.is_some() {
                    error!("<b>Two modules have been defined as <red>Operative System output</><b>. There can only be <cyan>one at a time</><b>.</>");
                    panic!("Two modules have been defined as Operative System output. There can only be one. Please check the logs more information.");
                }
                first_module = Some(module_id);
            }
        }

        // TYPE
        if module_type.as_str().is_none() {
            error!("<b>Module <red>type</> <b>not specified.</>");
            return Err(MissingField(String::from("type")));
        }

        let module_type = module_type.as_str().unwrap();
        let config = &module["config"];
        let name = config["name"].as_str();

        if let Some(name) = name {
            info!("  |_ name: {}", name);
        }

        let generated_module: Box<dyn Module> = match module_type {
            "oscillator" => {
                if !config.is_null() {
                    let sample_rate = config["sample_rate"].as_i64();
                    let amp = config["amplitude"].as_f64();
                    let freq = config["frequency"].as_f64();
                    let phase = config["phase"].as_f64();
                    let pwd = config["pwd"].as_f64();

                    let wave = match config["wave"].as_str() {
                        None => None,
                        Some(str) => match str {
                            "sin" | "sine" => Some(WaveShape::Sine),
                            "tri" | "triangle" => Some(WaveShape::Triangle),
                            "saw" => Some(WaveShape::Saw),
                            "sqr" | "square" => Some(WaveShape::Square),
                            "pulse" => {
                                let width: f32 = match pwd {
                                    Some(x) => x as f32,
                                    None => PI,
                                };
                                Some(WaveShape::Pulse(width))
                            }
                            &_ => None,
                        },
                    };

                    Box::new(
                        OscillatorBuilder::with_all_yaml_fmt(name, amp, freq, phase, wave, pwd)
                            .build()
                            .unwrap(),
                    )
                } else {
                    info!("No configuration found for oscillator");
                    Box::new(OscillatorBuilder::new().build().unwrap())
                }
            }

            "sum" => {
                let input_amount = config["input-amount"].as_i64();

                if input_amount.is_none() {
                    error!(
                        "<b>Invalid format or no <red>input amount</> <b>provided for sum module. ID: {}.</>",
                        module_id
                    );
                    return Err(MissingField(String::from("input-amount")));
                }

                let input_amount = input_amount.unwrap();

                let out_gain = &config["out-gain"];
                let in_1_gain = &config["in-1"];
                let in_2_gain = &config["in-2"];
                let in_3_gain = &config["in-3"];

                let items: Vec<Option<f64>> = [out_gain, in_1_gain, in_2_gain, in_3_gain]
                    .into_iter()
                    .map(|yaml| match yaml {
                        Yaml::Real(_) => yaml.as_f64(),
                        Yaml::Integer(_) => yaml.as_i64().map(|x| x as f64),
                        _ => None,
                    })
                    .collect();
                let (out_gain, in_1_gain, in_2_gain, in_3_gain) =
                    (items[0], items[1], items[2], items[3]);

                if input_amount <= 1 {
                    error!("<b><redInvalid amount</> <b>of inputs declared</>");
                    error!("  |_ id: {}", module_id);
                    return Err(InvalidInputAmount(input_amount));
                } else if input_amount == 2 {
                    Box::new(
                        Sum2InBuilder::with_all_yaml(name, out_gain, in_1_gain, in_2_gain)
                            .build()
                            .unwrap(),
                    )
                } else if input_amount == 3 {
                    Box::new(
                        Sum3InBuilder::with_all_yaml(
                            name, out_gain, in_1_gain, in_2_gain, in_3_gain,
                        )
                        .build()
                        .unwrap(),
                    )
                } else {
                    if in_1_gain.is_some() || in_2_gain.is_some() || in_3_gain.is_some() {
                        warn!("<b>For sum modules with a size greater than 3 is <yellow>not possible to specify the input gain</> <b>for each input. Instead, you have to specify it in the module itself.</>");
                        warn!("  * found in module with id: {}", module_id);
                    }

                    Box::new(
                        VarSumBuilder::with_all_yaml(name, input_amount, out_gain)
                            .build()
                            .unwrap(),
                    )
                }
            }
            "osc_debug" => Box::new(OscDebug::new(SAMPLE_RATE)),
            "pass_through" => Box::new(PassTrough::new()),

            _ => {
                error!("<b>Module type <red>not known</><b>. ID: {}.</>", module_id);
                return Err(UnknownType(module_type.to_string()));
            }
        };

        info!("  |_ type: {}", module_type);

        // ADD AUXILIARIES
        let mut auxiliaries: Vec<AuxInfo> = Vec::new();
        info!("  |_ looking for auxiliaries");

        let mut aux_count = 0;
        for aux in module["auxiliaries"].clone().into_iter() {
            aux_count += 1;
            let aux = &aux["aux"];

            let from_id = &aux["from-id"];
            let id_from = match from_id {
                Yaml::Real(_) => from_id.as_f64().map(|x| x as i64),
                Yaml::Integer(_) => from_id.as_i64(),

                Yaml::BadValue => {
                    warn!("<b>Missing <yellow>from-id</> <b>value.</>");
                    error!(
                        "<b>from-id parameter is <red>compulsory</> <b>for auxiliary inputs to know where the data goes to.</>"
                    );
                    return Err(MissingAuxFromId);
                }

                _ => {
                    error!("<b>Invalid format for <red>from-id</> <b>value.</>");
                    return Err(WrongFormat {
                        field_name: String::from("from-id"),
                        supported_format: String::from("i64"),
                    });
                }
            };

            let tag = &aux["linked-with"];
            let tag = match tag {
                Yaml::Real(_) => tag.as_f64().map(|x| x.to_string()),
                Yaml::Integer(_) => tag.as_i64().map(|x| x.to_string()),
                Yaml::Boolean(_) => tag.as_bool().map(|x| x.to_string()),
                Yaml::String(_) => tag.as_str().map(|x| x.to_string()),
                Yaml::BadValue => {
                    warn!("<b>Missing <yellow>linked-with</> <b>value.</>");
                    error!(
                    "<b>linked-with parameter is <red>compulsory</> <b>for auxiliary inputs to know to which parameter maps to.</>"
                );
                    return Err(MissingAuxTag);
                }
                _ => {
                    error!("<b>Invalid format for <red>linked-with</> <b>value.</>");
                    return Err(WrongFormat {
                        field_name: String::from("linked-with"),
                        supported_format: String::from("str"),
                    });
                }
            };

            let max = match &aux["max"] {
                Yaml::Real(_) => aux["max"].as_f64().map(|x| x as f32),
                Yaml::Integer(_) => aux["max"].as_i64().map(|x| x as f32),
                Yaml::BadValue => None, // not found
                _ => {
                    warn!("<b>Invalid format for <yellow>max</> <b>value.</>");
                    None
                }
            };

            let min = match &aux["min"] {
                Yaml::Real(_) => aux["min"].as_f64().map(|x| x as f32),
                Yaml::Integer(_) => aux["min"].as_i64().map(|x| x as f32),
                Yaml::BadValue => None, // not found
                _ => {
                    warn!("<b>Invalid format for <yellow>min</> <b>value.</>");
                    None
                }
            };

            let tag = tag.expect(
                "An auxiliary is not specifying 'linked-with' field. Please check the logs for more information.",
            );

            let from_id = id_from.expect("An auxiliary is missing the 'from-id' field. Please check the logs for more information.");

            info!("    |_ routing {} to module #{}", tag, from_id);

            auxiliaries.push(AuxInfo {
                from_module: from_id,
                linked_with: tag,
                max,
                min,
            });
        }

        if let Some(input_amount) = config["input-amount"].as_i64() {
            if aux_count < input_amount - 1 {
                // input amount specified - amount of directly routed inputs (one, currently)
                warn!("<b>A {} module has been detected not to have every input routed to an <yellow>auxiliary</><b>.</>", module_type);
                warn!("  |_ id: {}", module_id);
                warn!("  |_ aux count: {}", aux_count);
                warn!("  |_ input amt: {}", input_amount);
            }
        }

        module_chain.insert(
            module_id,
            ChainCell {
                from_module: (&module["input-from"]).as_i64(),
                module: generated_module,
                auxiliaries,
            },
        );
    }

    if first_module.is_none() {
        error!("<b>No module linked to <red>Operating System</><b>. Add field 'os-out: true' to the last element in the chain.</>");
        return Err(MissingOpSysOutput);
    }

    *first_module_index = first_module.unwrap();
    info!("First module's index: {}", first_module_index);

    Ok(module_chain)
}

pub fn buffer_from_yaml(file: &str, buffer_length: usize, sample_rate: i32) -> Vec<f32> {
    let mut first_module = 0i64;
    let mut module_chain = load_yaml(file, &mut first_module);

    info!("<b>Filling buffer:</>\n");
    fill_buffer(
        &mut module_chain.unwrap(),
        first_module,
        buffer_length,
        sample_rate,
    )
}

pub fn play_from_yaml(
    file: &str,
    signal_duration: i32,
    sample_rate: i32,
) -> Result<(), anyhow::Error> {
    let mut first_module = 0i64;
    let mut module_chain = load_yaml(file, &mut first_module);
    let mut wrapper_chain: LinkedList<Box<dyn ModuleWrapper>> = LinkedList::new();

    let ring_buffer: HeapRb<f32> = HeapRb::new(BATCH_SIZE_RT);
    let (prod, mut cpal_consumer) = ring_buffer.split();

    build_wrapper_chain(
        &mut module_chain.unwrap(),
        first_module,
        &mut wrapper_chain,
        prod,
    );

    let mut coordinator = CoordinatorEntity::new(sample_rate, wrapper_chain);
    coordinator.display_order();

    // CPAL CONFIGURATION
    let mut logger = simplelog::__private::paris::Logger::new();
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

    let mut next_value = move || cpal_consumer.pop().unwrap_or(0.0); // Unwrap or silence

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

    let mut count = 0;
    while count < (signal_duration as f32 * sample_rate as f32 / 1000.0) as i32 {
        if !coordinator.is_full() {
            coordinator.tick();
            count += 1;
        }
    }

    logger.done();

    Ok(())

    // TODO add a "to_module" field to the ChainCell struct which is calculated ()
    // during the load_yaml. Is read to generate the producer and consumer pair.
    // the modules without a "from_module" field are considered to be generators
    // and will start this process.
    // A generator module will be created and its producer defined.
    // Then we go to the next module and

    // TODO how do auxiliaries work in this context?

    // TODO consider ringbuf capacity. Test performance

    // TODO add a module with id -1 to the chain, which is the cpal output module.
    // TODO raise an exception if id -1 is used.

    // REFLECTIONS
    // The structure is highly efficient. The time complexity is lineal.
    // The delay of generating the whole chain is one sample per module in the
    // deepest (longest) chain in the structure. Even parallel chains such as
    // auxiliaries mean no extra time, at least for the structure. Obviously we
    // also have to consider the extra overhead of the calculations of each module.
    //
    // One possible performance improvement would be to find a way of getting
    // rid of such delay. (which only happens at the beginning of the playback
    // not in each module, so is not severe at all).

    // PERFORMANCE IMPROVEMENTS
    // An option for increasing performance is using threads for processing different
    // parts of the chain at the same time. If the chain had no branches (no
    // auxiliaries) the optimization would be creating more than one coordinator,
    // each one in charge of one part of the chain. Performance testing would be
    // necessary to get to a exact number, but let us use five modules per coordinator
    // as an example. If we had ten modules, they would split equally the work and
    // the overhead added by the coordinator entity would be reduced.
    // Actually, a coordinator would not be viable, as it has a clock in it, which
    // has to be universal to every module.
    //
    // Another possible improvement is to have a thread for each branch.
    // We would need to think of branches and junctions, where junctions should
    // be understood as modules where more than one module meet.
}

// An optimization with threads would not be possible as a recursive function does not
// have perspective of the whole structure.
/// Fill the whole buffer from the module chain structure.
fn fill_buffer(
    module_chain: &mut HashMap<i64, ChainCell>,
    current_pos: i64,
    buffer_size: usize,
    sample_rate: i32,
) -> Vec<f32> {
    let mut current_module = module_chain.remove(&current_pos).unwrap();
    let next_id = current_module.from_module;

    // AUXILIARIES
    let mut aux_list: Vec<AuxiliaryInput> = Vec::new();

    for aux_info in current_module.auxiliaries {
        let aux_buffer = fill_buffer(module_chain, aux_info.from_module, buffer_size, sample_rate);
        let aux = AuxInputBuilder::new(&aux_info.linked_with, AuxDataHolder::Batch(aux_buffer))
            .with_all_yaml(aux_info.max, aux_info.min)
            .build()
            .unwrap();

        aux_list.push(aux);
    }

    // GENERATE OR PROCESS BUFFER
    return if next_id.is_some() {
        // LINKER MODULE (PROCESS BUFFER) - RECURSIVE STEP

        let next_id = next_id.unwrap();
        let mut buffer = fill_buffer(module_chain, next_id, buffer_size, sample_rate);
        current_module
            .module
            .fill_buffer(&mut buffer, sample_rate, aux_list);
        buffer
    } else {
        // GENERATOR MODULE (CREATE BUFFER) - BASE CASE

        let mut buffer = vec![0.0f32; buffer_size];

        current_module
            .module
            .fill_buffer(&mut buffer, sample_rate, aux_list);
        buffer
    };
}

fn build_wrapper_chain(
    module_chain: &mut HashMap<i64, ChainCell>,
    current_pos: i64,
    wrapper_chain: &mut LinkedList<Box<dyn ModuleWrapper>>,
    producer: ModuleProducer,
) {
    // If current module has not
    let current_module = module_chain.remove(&current_pos).unwrap();
    let next_id = current_module.from_module;

    // AUXILIARIES
    let mut aux_list: Vec<AuxiliaryInput> = Vec::new();

    for aux_info in current_module.auxiliaries {
        let aux_id = aux_info.from_module;
        let rb: HeapRb<f32> = HeapRb::new(BATCH_SIZE_RT);
        let (prod, cons) = rb.split();

        let aux = AuxInputBuilder::new(&aux_info.linked_with, AuxDataHolder::RealTime(cons))
            .with_all_yaml(aux_info.max, aux_info.min)
            .build()
            .unwrap();
        build_wrapper_chain(module_chain, aux_id, wrapper_chain, prod);

        aux_list.push(aux);
    }

    if next_id.is_some() {
        // LINKER MODULE - RECURSIVE STEP
        let rb: HeapRb<f32> = HeapRb::new(BATCH_SIZE_RT);
        let (prod, cons) = rb.split();
        let wrapper = LinkerModuleWrapper::new(current_module.module, cons, producer, aux_list);

        // To ensure that the sample of the previous module is generated first
        // We fist add the AUXILIARY
        build_wrapper_chain(module_chain, next_id.unwrap(), wrapper_chain, prod);
        // and then the current module
        wrapper_chain.push_back(Box::new(wrapper));
    } else {
        // GENERATOR MODULE - BASE CASE
        let wrapper = GeneratorModuleWrapper::new(current_module.module, producer, aux_list);

        wrapper_chain.push_back(Box::new(wrapper));
    }
}
