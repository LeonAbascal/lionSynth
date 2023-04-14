use crate::bundled_modules::debug::*;
use crate::bundled_modules::*;
use crate::module::{AuxInputBuilder, AuxiliaryInput, Module};
use crate::SAMPLE_RATE;
use simplelog::{error, info};
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use yaml_rust::{YamlEmitter, YamlLoader};

struct ChainCell {
    id: i64,
    from_module: Option<i64>,
    to_module: i64,
    module: Box<dyn Module>,
    auxiliaries: Vec<AuxInfo>,
}

struct AuxInfo {
    from_module: i64,
    linked_with: String,
    max: Option<i64>,
    min: Option<i64>,
}

pub fn module_chain_from_yaml(file: &str, buffer_length: usize) -> Vec<f32> {
    println!(); // Logger cleanspace

    info!("<b>Loading data from <red>layouts/{}</><b>.</>", file);
    let path = format!("layouts/{}", file);
    let yaml = &fs::read_to_string(path).unwrap();

    let doc = YamlLoader::load_from_str(yaml).unwrap();
    let doc = &doc[0];

    let version = *&doc["version"].as_f64().unwrap_or(0.0);
    let layout = &doc["layout"];

    if version != 0.3f64 {
        error!("<b>Please use the <red>latest YAML</> <b>version.</>");
        panic!();
    } else {
        info!(
            "<b>Using <magenta>YAML parsing</> version: <b><cyan>{}</>",
            version
        );
    }

    info!("<b>Creating module chain.</>");
    let mut module_chain: HashMap<i64, ChainCell> = HashMap::new();

    for module in layout.clone().into_iter() {
        let module = &module["module"];
        let module_type = &module["type"];
        let module_id = module["id"].as_i64().unwrap();

        if module_type.as_str().is_none() {
            error!("<b>Module <red>type</> <b>not specified.</>");
            panic!();
        }

        let module_type = module_type.as_str().unwrap();
        let config = &module["config"];
        let name = config["name"].as_str();

        let mut generated_module: Box<dyn Module> = match module_type {
            "oscillator" => {
                let sample_rate = config["sample_rate"].as_i64();
                let amp = config["amplitude"].as_f64();
                let freq = config["frequency"].as_f64();
                let phase = config["phase"].as_f64();

                Box::new(
                    OscillatorBuilder::new()
                        .with_all_yaml_fmt(name, sample_rate, amp, freq, phase)
                        .build()
                        .unwrap(),
                )
            }

            "osc_debug" => Box::new(OscDebug::new(SAMPLE_RATE)),
            "pass_through" => Box::new(PassTrough::new()),

            _ => {
                error!("<b>Module type <red>not found</><b>. ID: {}</>", module_id);
                continue;
            }
        };

        let mut id_to = (&module["output-to"]).as_i64();

        // ADD AUXILIARIES
        let mut auxiliaries: Vec<AuxInfo> = Vec::new();
        for aux in module["auxiliaries"].clone().into_iter() {
            let aux = &aux["aux"];

            auxiliaries.push(AuxInfo {
                from_module: aux["id-from"].as_i64().unwrap(),
                linked_with: aux["linked-with"].as_str().unwrap().to_string(),
                max: aux["max"].as_i64(),
                min: aux["min"].as_i64(),
            });
        }

        match id_to {
            Some(id_to) => {
                module_chain.insert(
                    module_id,
                    ChainCell {
                        id: module_id,
                        from_module: (&module["input-from"]).as_i64(),
                        to_module: (&module)["output-to"].as_i64().unwrap(),
                        module: generated_module,
                        auxiliaries,
                    },
                );
            }

            _ => {
                error!("<b>Missing module <red>ID</><b>.</>");
                panic!();
            }
        }
    }

    let first_module_index = module_chain.iter().find_map(|(&index, cell)| {
        if cell.to_module == -1 {
            Some(index)
        } else {
            None
        }
    });

    if first_module_index.is_none() {
        error!("<b>No module linked to <red>Operating System</><b>. Set ID -1 to the last module in the chain.</>");
        panic!();
    }

    let first_module_index = first_module_index.unwrap();

    let mut current_module = module_chain.get(&first_module_index).unwrap();

    info!("<b>Creating buffer.</>");
    let mut buffer: Vec<f32> = vec![0.0f32; buffer_length as usize];

    // TODO REMOVE (PRINTS ALL ELEMENTS)
    // for (&index, cell) in module_chain.iter() {
    //     info!("index: {}", index);
    //     info!("  |_ name: {}", cell.module.get_name());
    // }

    info!("<b>Filling buffer:</>\n");
    let mut buffer =
        generate_from_module_chain(&mut module_chain, first_module_index, buffer_length);

    exit(0);
    return buffer;
}

// TODO auxiliaries
fn generate_from_module_chain(
    module_chain: &mut HashMap<i64, ChainCell>,
    current_pos: i64,
    buffer_size: usize,
) -> Vec<f32> {
    let current_module_borrow = module_chain.get(&current_pos).unwrap();
    let condition = current_module_borrow.from_module.is_some();

    return if condition {
        // LINKER MODULE

        let next_id = current_module_borrow.from_module.unwrap();
        let mut current_module = module_chain.remove(&current_pos).unwrap();
        let mut buffer = generate_from_module_chain(module_chain, next_id, buffer_size);
        current_module.module.fill_buffer(&mut buffer);
        buffer
    } else {
        // GENERATOR MODULE

        let mut buffer = vec![0.0f32; buffer_size];
        let mut current_module = module_chain.remove(&current_pos).unwrap();
        let mut aux_list: Vec<AuxiliaryInput> = Vec::new();
        for aux in current_module.auxiliaries {
            let aux_buffer = generate_from_module_chain(module_chain, aux.from_module, buffer_size);
            let aux = AuxInputBuilder::new(&aux.linked_with, aux_buffer)
                .build()
                .unwrap();

            aux_list.push(aux);
        }

        current_module.module.fill_buffer(&mut buffer);
        buffer
    };
}

// TODO
fn check_duplicated() {}
