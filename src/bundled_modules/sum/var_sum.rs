//! A **sum module** adds each of the input signals.
//!
//! # Behavior
//! Since a module can only have one regular input, parameters are used to assist the task,
//! so what this module actually does is to add the values of the input and the auxiliaries.
//!
//! # Warning
//! When using these modules you should be careful with the volume of each of the inputs, as they
//! will easily clip if a proper gain stage is not performed before or after the sum. Values may
//! not clip in every type of module, but will surely clip once hit the output of the operating
//! system.

use super::*;
use crate::module::{Module, Parameter, ParameterBuilder};

/// The [VarSum] will let you create a sum module with any amount of modules.
///
/// The drawback of this type of sum module is that it is not currently possible to
/// adjust the gain of each of the inputs. Instead, it will be at user's charge.
pub struct VarSum {
    name: String,
    in_count: u32,
    inputs: Vec<Parameter>,
    out_gain: Parameter,
}

impl Module for VarSum {
    fn behaviour(&self, in_data: f32, _time: f32) -> f32 {
        let mut result = in_data;

        for in_value in self.inputs.iter() {
            result += in_value.get_value();
        }

        result * self.out_gain.get_value()
    }

    fn get_parameters(&self) -> Option<Vec<&Parameter>> {
        let mut parameters: Vec<&Parameter> = Vec::new();

        self.inputs.iter().for_each(|p| parameters.push(&p));

        parameters.push(&self.out_gain);
        Some(parameters)
    }

    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>> {
        let mut parameters: Vec<&mut Parameter> = Vec::new();

        self.inputs.iter_mut().for_each(|p| parameters.push(p));
        Some(parameters)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub struct VarSumBuilder {
    name: Option<String>,
    in_count: Option<u32>,
    out_gain: Option<f32>,
}

impl VarSumBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            in_count: None,
            out_gain: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_output_gain(mut self, out_gain: f32) -> Self {
        self.out_gain = Some(out_gain);
        self
    }

    pub fn input_amt(mut self, in_count: u32) -> Self {
        self.in_count = Some(in_count);
        self
    }

    pub fn build(mut self) -> Result<VarSum, String> {
        let in_count = self.in_count.unwrap_or(2);
        let out_gain = self.out_gain.unwrap_or(1.0);

        let name = match self.name {
            Some(name) => format!("{} {}Sum", name, in_count),
            None => format!("{} Sum", in_count),
        };

        let mut inputs = vec![];
        for i in 0..in_count {
            let param = ParameterBuilder::new(format!("in{}", i))
                .with_min(AUDIO_RANGE_BOT)
                .with_default(0.0)
                .with_max(AUDIO_RANGE_TOP)
                .build()
                .unwrap();
            inputs.push(param);
        }

        Ok(VarSum {
            name,
            in_count,
            inputs,
            out_gain: ParameterBuilder::new("out_gain".to_string())
                .with_max(OVER_GAIN)
                .with_default(out_gain)
                .with_min(MIN_GAIN)
                .build()
                .unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod sum_tests {
        use super::*;
    }
}
