use crate::module::{Module, Parameter};

pub struct Sum2In {
    name: String,
    second_input: Parameter,
    out_gain: f32,
    in1_gain: f32,
    in2_gain: f32,
}

pub struct Sum3In {
    name: String,
    second_input: Parameter,
    third_input: Parameter,
    out_gain: f32,
    in1_gain: f32,
    in2_gain: f32,
    in3_gain: f32,
}

impl Sum2In {
    fn get_in2(&self) -> f32 {
        self.second_input.get_value()
    }
}

impl Module for Sum2In {
    fn behaviour(&self, in_data: f32, _time: f32) -> f32 {
        let in_1 = in_data * self.in1_gain;
        let in_2 = self.get_in2() * self.in2_gain;

        (in_1 + in_2) * self.out_gain
    }

    fn get_parameters(&self) -> Option<Vec<&Parameter>> {
        todo!()
    }

    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>> {
        todo!()
    }

    fn get_name(&self) -> String {
        todo!()
    }
}

pub struct Sum2InBuilder {
    name: Option<String>,
    in_1: Option<f32>,
    /// Default value for in_2 gain
    in_2: Option<f32>,
}
