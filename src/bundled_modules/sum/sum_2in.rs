use crate::module::{Module, Parameter, ParameterBuilder};

/// Sums the data from the input module and the auxiliary parameter.
/// # Input data
/// Input values come from auxiliaries and will get clipped to [1,-1] before adjusting
/// the input within the module, so be sure modules don't have a too hot output.
pub struct Sum2In {
    name: String,
    second_input: Parameter,
    out_gain: Parameter,
    in1_gain: Parameter,
    in2_gain: Parameter,
}

pub struct Sum3In {
    name: String,
    second_input: Parameter,
    third_input: Parameter,
    out_gain: Parameter,
    in1_gain: Parameter,
    in2_gain: Parameter,
    in3_gain: Parameter,
}

impl Sum2In {
    /// Gets the value from the parameter
    pub fn get_in2(&self) -> f32 {
        self.second_input.get_value()
    }
    pub fn set_in1_gain(&mut self, gain: f32) {
        self.in1_gain.set(gain);
    }

    pub fn set_in2_gain(&mut self, gain: f32) {
        self.in2_gain.set(gain);
    }

    pub fn set_out_gain(&mut self, gain: f32) {
        self.out_gain.set(gain);
    }
}

impl Module for Sum2In {
    fn behaviour(&self, in_data: f32, _time: f32) -> f32 {
        let in_1 = in_data * self.in1_gain.get_value();
        let in_2 = self.get_in2() * self.in2_gain.get_value();

        println!("VALUE:");
        println!("  |_ IN1: {}", in_data);
        println!("    |_ gain: {}", self.in1_gain.get_value());
        println!("    |_ value: {}", in_1);
        println!("  |_ IN2: {}", self.get_in2());
        println!("    |_ gain: {}", self.in2_gain.get_value());
        println!("    |_ value: {}", in_2);
        println!("  |_ OUT: {}", self.out_gain.get_value());

        (in_1 + in_2) * self.out_gain.get_value()
    }

    fn get_parameters(&self) -> Option<Vec<&Parameter>> {
        Some(vec![
            &self.second_input,
            &self.in1_gain,
            &self.in2_gain,
            &self.out_gain,
        ])
    }

    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>> {
        Some(vec![
            &mut self.second_input,
            &mut self.in1_gain,
            &mut self.in2_gain,
            &mut self.out_gain,
        ])
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub struct Sum2InBuilder {
    name: Option<String>,
    in_1: Option<f32>,
    /// Default value for in_2 gain
    in_2: Option<f32>,
    out_gain: Option<f32>,
}

impl Sum2InBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            in_1: None,
            in_2: None,
            out_gain: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_gain(mut self, gain: f32) -> Self {
        self.out_gain = Some(gain);
        self
    }

    pub fn with_gain_1(mut self, gain: f32) -> Self {
        self.in_1 = Some(gain);
        self
    }

    pub fn with_gain_2(mut self, gain: f32) -> Self {
        self.in_2 = Some(gain);
        self
    }

    pub fn build(self) -> Result<Sum2In, String> {
        let over_gain = 2.0; // Maximum gain, to compensate low levels

        let name = match self.name {
            Some(name) => format!("{} Sum 2in", name),
            None => format!("Sum 2in"),
        };
        let out_gain = self.out_gain.unwrap_or(1.0);
        let in_1_gain = self.in_1.unwrap_or(1.0);
        let in_2_gain = self.in_2.unwrap_or(1.0);

        Ok(Sum2In {
            name,
            second_input: ParameterBuilder::new("in2".to_string())
                .with_min(-1.0)
                .with_max(1.0)
                .build()
                .unwrap(),

            out_gain: ParameterBuilder::new("out_gain".to_string())
                .with_max(over_gain)
                .with_default(out_gain)
                .with_min(0.0)
                .build()
                .unwrap(),

            in1_gain: ParameterBuilder::new("in_1_gain".to_string())
                .with_max(over_gain)
                .with_default(in_1_gain)
                .with_min(0.0)
                .build()
                .unwrap(),

            in2_gain: ParameterBuilder::new("in_2_gain".to_string())
                .with_max(over_gain)
                .with_default(in_2_gain)
                .with_min(0.0)
                .build()
                .unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod sum_2_in_tests {
        use super::*;
        use crate::bundled_modules::debug::OscDebug;
        use crate::module::{AuxDataHolder, AuxInputBuilder};
        use crate::SAMPLE_RATE;
        use log::info;

        fn get_sum_module() -> Sum2In {
            Sum2InBuilder::new().build().unwrap()
        }

        #[test]
        fn test_get_in_2() {
            const BUFFER_SIZE: usize = 10;

            let mut in1_osc = OscDebug::new(SAMPLE_RATE);
            let mut in2_osc = OscDebug::new(SAMPLE_RATE);
            let mut sum = get_sum_module();

            let mut buffer1 = vec![0.0f32; BUFFER_SIZE];
            let mut buffer2 = vec![0.0f32; BUFFER_SIZE];

            in1_osc.fill_buffer(&mut buffer1, vec![]);
            in2_osc.fill_buffer(&mut buffer2, vec![]);

            assert_eq!(buffer1, buffer2);

            sum.fill_buffer(
                &mut buffer1,
                vec![AuxInputBuilder::new("in2", AuxDataHolder::Batch(buffer2))
                    .build()
                    .unwrap()],
            );
        }
    }
}
