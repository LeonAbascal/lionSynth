use super::*;
use crate::module::{Module, Parameter, ParameterBuilder};

pub struct Sum3In {
    name: String,
    second_input: Parameter,
    third_input: Parameter,
    out_gain: Parameter,
    in1_gain: Parameter,
    in2_gain: Parameter,
    in3_gain: Parameter,
}

impl Sum3In {
    /// Gets the value from the parameter
    pub fn get_in2(&self) -> f32 {
        self.second_input.get_value()
    }
    pub fn get_in3(&self) -> f32 {
        self.third_input.get_value()
    }

    pub fn set_in1_gain(&mut self, gain: f32) {
        self.in1_gain.set(gain);
    }
    pub fn set_in2_gain(&mut self, gain: f32) {
        self.in2_gain.set(gain);
    }
    pub fn set_in3_gain(&mut self, gain: f32) {
        self.in3_gain.set(gain);
    }

    pub fn set_out_gain(&mut self, gain: f32) {
        self.out_gain.set(gain);
    }
}

impl Module for Sum3In {
    fn behaviour(&self, in_data: f32, _time: f32) -> f32 {
        let in_1 = in_data * self.in1_gain.get_value();
        let in_2 = self.get_in2() * self.in2_gain.get_value();
        let in_3 = self.get_in3() * self.in3_gain.get_value();

        (in_1 + in_2 + in_3) * self.out_gain.get_value()
    }

    fn get_parameters(&self) -> Option<Vec<&Parameter>> {
        Some(vec![
            &self.second_input,
            &self.third_input,
            &self.in1_gain,
            &self.in2_gain,
            &self.in3_gain,
            &self.out_gain,
        ])
    }

    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>> {
        Some(vec![
            &mut self.second_input,
            &mut self.third_input,
            &mut self.in1_gain,
            &mut self.in2_gain,
            &mut self.in3_gain,
            &mut self.out_gain,
        ])
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

pub struct Sum3InBuilder {
    name: Option<String>,
    /// Default value for in_1 gain
    in_1: Option<f32>,
    /// Default value for in_2 gain
    in_2: Option<f32>,
    /// Default value for in_3 gain
    in_3: Option<f32>,
    /// Output gain
    out_gain: Option<f32>,
}

impl Sum3InBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            in_1: None,
            in_2: None,
            in_3: None,
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

    pub fn with_gain_in1(mut self, gain: f32) -> Self {
        self.in_1 = Some(gain);
        self
    }

    pub fn with_gain_in2(mut self, gain: f32) -> Self {
        self.in_2 = Some(gain);
        self
    }

    pub fn with_gain_in3(mut self, gain: f32) -> Self {
        self.in_3 = Some(gain);
        self
    }

    pub fn with_all_yaml(
        name: Option<&str>,
        out_gain: Option<f64>,
        in_1_gain: Option<f64>,
        in_2_gain: Option<f64>,
        in_3_gain: Option<f64>,
    ) -> Self {
        Self {
            name: name.map(|x| x.to_string()),
            out_gain: out_gain.map(|x| x as f32),
            in_1: in_1_gain.map(|x| x as f32),
            in_2: in_2_gain.map(|x| x as f32),
            in_3: in_3_gain.map(|x| x as f32),
        }
    }

    pub fn build(self) -> Result<Sum3In, String> {
        let name = match self.name {
            Some(name) => format!("{} Sum 3in", name),
            None => format!("Sum 3in"),
        };

        let out_gain = self.out_gain.unwrap_or(1.0);
        let in_1_gain = self.in_1.unwrap_or(1.0);
        let in_2_gain = self.in_2.unwrap_or(1.0);
        let in_3_gain = self.in_3.unwrap_or(1.0);

        Ok(Sum3In {
            name,
            second_input: ParameterBuilder::new("in2".to_string())
                .with_min(AUDIO_RANGE_BOT)
                .with_max(AUDIO_RANGE_TOP)
                .build()
                .unwrap(),

            third_input: ParameterBuilder::new("in3".to_string())
                .with_min(AUDIO_RANGE_BOT)
                .with_max(AUDIO_RANGE_TOP)
                .build()
                .unwrap(),

            out_gain: ParameterBuilder::new("out_gain".to_string())
                .with_max(OVER_GAIN)
                .with_default(out_gain)
                .with_min(MIN_GAIN)
                .build()
                .unwrap(),

            in1_gain: ParameterBuilder::new("in_1_gain".to_string())
                .with_max(OVER_GAIN)
                .with_default(in_1_gain)
                .with_min(MIN_GAIN)
                .build()
                .unwrap(),

            in2_gain: ParameterBuilder::new("in_2_gain".to_string())
                .with_max(OVER_GAIN)
                .with_default(in_2_gain)
                .with_min(MIN_GAIN)
                .build()
                .unwrap(),

            in3_gain: ParameterBuilder::new("in_3_gain".to_string())
                .with_max(OVER_GAIN)
                .with_default(in_3_gain)
                .with_min(MIN_GAIN)
                .build()
                .unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bundled_modules::debug_modules::*;
    use crate::module::{AuxDataHolder, AuxInputBuilder};
    use crate::SAMPLE_RATE;

    #[test]
    fn sum3_module() {
        const BUFFER_SIZE: usize = 10;

        let mut in1_osc = OscDebug::new(SAMPLE_RATE);
        let mut in2_osc = OscDebug::new(SAMPLE_RATE);
        let mut in3_osc = OscDebug::new(SAMPLE_RATE);
        let mut sum = Sum3InBuilder::new().build().unwrap();

        let mut buffer1 = vec![0.0f32; BUFFER_SIZE];
        let mut buffer2 = vec![0.0f32; BUFFER_SIZE];
        let mut buffer3 = vec![0.0f32; BUFFER_SIZE];

        in1_osc.fill_buffer(&mut buffer1, vec![]);
        in2_osc.fill_buffer(&mut buffer2, vec![]);
        in3_osc.fill_buffer(&mut buffer3, vec![]);

        assert_eq!(buffer1, buffer2);
        assert_eq!(buffer1, buffer3);

        sum.fill_buffer(
            &mut buffer1,
            vec![
                AuxInputBuilder::new("in2", AuxDataHolder::Batch(buffer2))
                    .with_max(1.0)
                    .with_min(-1.0)
                    .build()
                    .unwrap(),
                AuxInputBuilder::new("in3", AuxDataHolder::Batch(buffer3))
                    .with_max(1.0)
                    .with_min(-1.0)
                    .build()
                    .unwrap(),
            ],
        );

        let deterministic_buffer = vec![
            0.0, 0.18794492, 0.37515163, 0.56088424, 0.7444135, 0.9250181, 1.1019888, 1.2746301,
            1.4422638, 1.604231,
        ];

        assert_eq!(deterministic_buffer, buffer1);
    }
}
