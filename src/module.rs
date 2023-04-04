use simplelog::{info, warn, TermLogger};

/// A **linker module** is a module able to connect into another. An example of linker module
/// would be an effect module or an [ADSR](https://en.wikipedia.org/wiki/Envelope_(music)) module.
/// An example of **not** linker module would be an generator module, which does not use any
/// input sample but instead generates one.
pub struct LinkerModule {
    // TODO
}

// TODO: revisit
/// Modules are the building blocks of a modular synthesizer, its essence. They are defined by
/// their behavior which can be modified with [Parameter].
///
/// # How it works
/// Each module is able to receive and retrieve a buffer of any size. Data (samples) is represented
/// in a [f32] format, and the module will modify it. For such, it will be calling the [behaviour](fn@Module::behaviour)
/// method, which is the only one you need to override. I do not recommend overriding the rest
/// of the methods.
///
/// # The tick
/// The tick is the way in which the module keeps track of time. Not every module needs it, but is
/// generally useful.
///
/// # The parameters
/// [Parameter] are what change the behaviour of the module in a specific moment.
/// TODO: finish doc
pub trait Module {
    // fn get_sample(&self); // real time behaviour?

    /// Fills the input buffer with new information. It may generate or modify the buffer.
    ///
    /// It also sets the clock forward and calls every function that needs to be updated on every
    /// tick, such as the [behavior](fn@Module::behaviour) or the update of the parameters.

    // TODO stereo
    fn fill_buffer_w_aux(
        &mut self,
        buffer: &mut Vec<f32>,
        auxiliaries: Option<Vec<&mut AuxiliaryInput>>,
    ) {
        warn!("<b>A <u>custom implementation</><b> for buffer filling with auxiliary inputs is recommended for better <yellow>performance</><b>.</>");

        // AUXILIARY INPUTS MANAGEMENT STARTS HERE
        // Map parameters
        let mut temp_aux: Vec<Option<AuxiliaryInput>> =
            Vec::with_capacity(self.get_parameter_count());

        if let Some(auxiliaries) = auxiliaries {
            for param in self.get_parameters() {
                if let Some(x) = auxiliaries.iter().find(|aux| aux.tag == param.tag) {
                    temp_aux.push(Some((**x).clone()));
                } else {
                    temp_aux.push(None);
                }
            }
        }

        #[cfg(feature = "verbose_modules")]
        {
            info!("<b>AUXILIARY LIST: {} items</>", temp_aux.len());
            for item in temp_aux.iter() {
                if item.is_none() {
                    info!("  |_ <red>No item</>");
                } else {
                    info!("  |_ <green>{} aux found</>", item.as_ref().unwrap().tag);
                }
            }
            println!();
        }

        // Create closure
        let mut pop_auxiliaries = move || -> Vec<Option<f32>> {
            let mut values = Vec::with_capacity((&temp_aux).len());

            for item in temp_aux.iter_mut() {
                match item {
                    Some(x) => values.push(item.as_mut().unwrap().pop()),
                    None => values.push(None),
                }
            }

            values
        };
        // AUXILIARY INPUTS MANAGEMENT ENDS HERE

        #[cfg(feature = "module_values")]
        let mut count = 0;

        for item in buffer {
            // Pre-processing operations
            // Buffer processing
            *item = self.behaviour(*item);

            let parameters = self.get_parameters_mutable();
            let mut new_values = pop_auxiliaries();
            new_values.reverse(); // We reverse it because later we use pop function

            for parameter in parameters {
                let previous_value = parameter.get_value();

                match new_values.pop().unwrap_or(None) {
                    Some(new_value) => {
                        #[cfg(feature = "module_values")]
                        info!(
                            "<b>Updating parameter <blue>{}</> <b>with {}</>",
                            parameter.tag, new_value
                        );
                        parameter.set(new_value)
                    }
                    None => parameter.set(previous_value),
                }
            }

            // POST-PROCESSING OPERATIONS
            self.inc_clock();

            #[cfg(feature = "module_values")]
            {
                count = (count + 1) % 10;
                info!("<b>[ {} ]</> {}", count, item);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut Vec<f32>) {
        self.fill_buffer_w_aux(buffer, None);
    }

    /// Defines the behaviour of the module. Is it going to generate data? Is it going to clip the
    /// data under a threshold? Here is where the magic happens. The **behaviour is what defines
    /// a module.**
    /// # Arguments
    /// * `in_data`: the sample to modify, if any. Won't use it if creating a generator module.
    /// # Returns
    /// A generated or modified sample.
    fn behaviour(&self, in_data: f32) -> f32;

    /// Adds a parameter to the list of parameters. If the tag is already in the list,
    /// the operation gets rejected.
    fn add_parameter(&mut self, in_parameter: Parameter) -> Result<(), String> {
        let parameters = self.get_parameters_mutable();

        let tag = &in_parameter.tag;
        let res = parameters.into_iter().find(|p| &p.tag == tag);

        if res.is_none() {
            parameters.push(in_parameter);
        } else {
            return Err("Parameter already exists".to_string());
        }

        Ok(())
    }

    /// Retrieves a **mutable** parameter given its tag, if exists.
    fn get_parameter_mutable(&mut self, tag: &str) -> Option<&mut Parameter> {
        self.get_parameters_mutable()
            .into_iter()
            .find(|p| p.tag == tag)
    }

    /// Retrieves a *non mutable* parameter given its tag, if exists. There is a mutable
    /// alternative as well.
    ///
    /// # Shortcut methods
    /// I **strongly** recommend adding shortcut methods for your own modules, being this the
    /// helper method for such.
    ///
    /// ## Example
    /// You can find find a real implementation in the
    /// [Oscillator](struct@crate::bundled_modules::Oscillator) module, **implementation section**.
    /// ```rust
    /// pub fn get_name_of_param(&self) -> f32 { // All parameters should return f32
    ///     self.get_parameter("parameter_tag").unwrap().get_value() // Hiding the operation
    /// }
    ///
    /// let current_value = module.get_name_of_param();
    /// ```
    fn get_parameter(&self, tag: &str) -> Option<&Parameter> {
        self.get_parameters().into_iter().find(|p| p.tag == tag)
    }

    /// Gets all parameters in the list. Used to enforce the presence of a parameter
    /// list in every module struct.
    fn get_parameters(&self) -> &Vec<Parameter>;
    /// Gets all mutable parameters in the list.
    fn get_parameters_mutable(&mut self) -> &mut Vec<Parameter>;

    fn get_parameter_count(&self) -> usize {
        self.get_parameters().len()
    }

    /// Will define how the clock goes forward. Useful for timed operations. Must increase by one
    /// and start at zero to work properly with auxiliary inputs
    fn inc_clock(&mut self); // TODO: consider making the clock common with an associated function (class method) or a sync-er structure in the main flow
    fn get_clock(&self) -> f32; // TODO: remove? tick already enforces the clock in the struct
}

/// Parameters are what control the behaviour of a module. For example, in an oscillator, some
/// parameters such as amplitude, phase or frequency are very desirable to be modified. For such
/// reason, we may create parameters that get linked to the behaviour of each module as an
/// interface for modifying such values from the main flow of the program.
///
/// # Usage
/// In any case, if you want to use parameters, please refer to the [ParameterBuilder], which
/// provides a modular builder for creating parameters.
#[derive(Debug, PartialEq)]
pub struct Parameter {
    /// Maximum value that the parameter can reach.
    max: f32,
    /// Minimum value that the parameter can reach.
    min: f32,
    /// The size of the increment, in other words, how big the step is.
    step: f32,
    /// The starting (or default) value of the parameter.
    default: f32,
    /// The runtime value of the parameter.
    current: f32,
    /// The tag of the parameter. Works as identifier to distinguish it from the other
    /// parameters of a module.
    tag: String,
}

/// A parameter of a module. To create one, please refer to [ParameterBuilder].
impl Parameter {
    pub fn get_tag(&self) -> &String {
        &self.tag
    }
    pub fn get_value(&self) -> f32 {
        self.current
    }

    /// Sets the value of a parameter
    pub fn set(&mut self, value: f32) {
        if value < self.max || value > self.min {
            self.current = value;
        }
    }

    /// Increases the value of the parameter upon maximum.
    pub fn inc(&mut self) {
        // if value exceeds the maximum, keep the max value.
        if self.current + self.step > self.max {
            self.current = self.max;

        // otherwise, keep increasing it
        } else {
            self.current += self.step;
        }
    }

    /// Decreases the value of the parameter upon minimum.
    pub fn dec(&mut self) {
        // if value exceeds the minimum, keep the min value.
        if self.current - self.step < self.min {
            self.current = self.min;

        // otherwise, keep lowering it
        } else {
            self.current -= self.step;
        }
    }
}

/// A builder pattern to create parameters in a modular fashion.
/// # Example
///
pub struct ParameterBuilder {
    max: Option<f32>,
    min: Option<f32>,
    step: Option<f32>,
    default: Option<f32>,
    tag: String,
}

impl ParameterBuilder {
    pub fn new(tag: String) -> Self {
        Self {
            max: None,
            min: None,
            step: None,
            default: None,
            tag,
        }
    }

    pub fn with_max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }
    pub fn with_min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn with_default(mut self, default: f32) -> Self {
        self.default = Some(default);
        self
    }

    pub fn build(self) -> Result<Parameter, String> {
        let max = self.max.unwrap_or(1.0);
        let min = self.min.unwrap_or(0.0);
        let step = self.step.unwrap_or(0.1);
        let default = self.default.unwrap_or(0.0);
        let current = default.clone();
        let tag = self.tag;

        if max <= min {
            return Err("Non valid max/min range.".to_string());
        }

        if default > max || default < min {
            return Err("Default value is out of range.".to_string());
        }

        // This is not technically an error - but it is simply stupid (or just a slip-up).
        // Therefore, it deserves a warning rather than an error but I'll keep it anyway
        if step > (max - min) {
            return Err(
                "Step too small. Can not be smaller than the difference between".to_string(),
            );
        }

        Ok(Parameter {
            max,
            min,
            step,
            default,
            current,
            tag,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AuxiliaryInput {
    tag: String,
    buffer: Vec<f32>,
    max: f32,
    min: f32,
}

impl AuxiliaryInput {
    pub fn get_tag(&self) -> String {
        self.tag.to_string()
    }

    /// Pops the latest value of the auxiliary input. Additionally, it performs a translation
    /// from the values ranging from -1 to 1 that every module should output into the max and
    /// min values specified when built.
    fn pop(&mut self) -> Option<f32> {
        match self.buffer.pop() {
            Some(x) => {
                let size = self.max - self.min;
                let axis = (self.max + self.min) / 2.0;

                let x = ((size * x + size) / 2.0) + axis;
                Some(x)
            }
            None => None,
        }
    }
}

impl AuxiliaryInput {}

/// Auxiliary Input Builder
pub struct AuxInputBuilder {
    tag: String,
    buffer: Vec<f32>,
    max: Option<f32>,
    min: Option<f32>,
}

impl AuxInputBuilder {
    pub fn new(tag: String, buffer: Vec<f32>) -> Self {
        Self {
            tag,
            buffer,
            max: None,
            min: None,
        }
    }

    pub fn with_max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }

    pub fn build(self) -> AuxiliaryInput {
        AuxiliaryInput {
            tag: self.tag,
            buffer: self.buffer,
            max: self.max.unwrap_or(0.0),
            min: self.min.unwrap_or(1.0),
        }
    }
}

#[cfg(test)]
mod parameter_builder_tests {
    use crate::module::{Parameter, ParameterBuilder};
    use log::info;
    use simplelog::__private::paris::Logger;

    fn get_logger() -> Logger<'static> {
        Logger::new()
    }

    #[test]
    fn test_empty() {
        let mut logger = get_logger();

        logger.info("<b>Running test for parameter builder with no arguments</>");

        let tested_param = ParameterBuilder::new(String::from("test")).build().unwrap();
        let testing_param = Parameter {
            max: 1.0,
            min: 0.0,
            step: 0.1,
            default: 0.0,
            current: 0.0,
            tag: "test".to_string(),
        };

        assert_eq!(
            tested_param, testing_param,
            "Empty constructor for Parameter Builder failed"
        );

        logger.success("<b>Test pass</>");
    }

    #[test]
    fn test_with_all_args() {
        let mut logger = get_logger();
        logger.info("<b>Running test for parameter builder with all arguments</>");

        let tested_param = ParameterBuilder::new(String::from("test"))
            .with_max(2.0)
            .with_min(1.0)
            .with_default(1.5)
            .with_step(0.3)
            .build()
            .unwrap();

        let testing_param = Parameter {
            max: 2.0,
            min: 1.0,
            step: 0.3,
            default: 1.5,
            current: 1.5,
            tag: "test".to_string(),
        };

        assert_eq!(
            tested_param, testing_param,
            "Constructor with all arguments for Parameter Builder failed"
        );

        logger.success("<b>Test pass</>");
    }

    #[test]
    #[should_panic]
    fn test_invalid_range_greater() {
        ParameterBuilder::new(String::from("test"))
            .with_min(1.0)
            .with_max(0.0)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_range_equal() {
        ParameterBuilder::new(String::from("test"))
            .with_min(0.0)
            .with_max(0.0)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_default_min() {
        ParameterBuilder::new(String::from("test"))
            .with_min(1.0)
            .with_default(0.5)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_default_max() {
        ParameterBuilder::new(String::from("test"))
            .with_max(0.0)
            .with_default(0.5)
            .build()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_step() {
        ParameterBuilder::new(String::from("test"))
            .with_max(1.0)
            .with_min(0.0)
            .with_step(1.5)
            .build()
            .unwrap();
    }
}
