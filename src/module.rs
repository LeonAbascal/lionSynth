use ringbuf::{Consumer, Producer, SharedRb};
use simplelog::{error, info, warn};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

pub type ModuleConsumer = Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
pub type ModuleProducer = Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;

fn pop_auxiliaries(
    auxiliaries: &mut Vec<AuxiliaryInput>,
    current_values: HashMap<String, f32>,
) -> HashMap<String, f32> {
    let result: HashMap<String, f32>;

    // TODO use current values
    result = auxiliaries
        .iter_mut()
        .map(|aux| {
            let tag = aux.tag.clone(); // Gets the parameter tag is associated with
            let value = match aux.pop() {
                // Gets the next sample in the vector
                Some(value) => value,
                None => {
                    let prev_value = *current_values.get(&tag).unwrap();
                    warn!("<b>Values of auxiliary list <yellow>exhausted</><b>. It is perfectly normal for the first samples of the chain.</>");
                    info!("Defaulting to previous value: {}", prev_value);
                    prev_value // Returns the previous value
                }
            };

            (tag, value) // Returns the generated pair
        })
        .collect();

    result
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
///
/// # Real time vs batch processing
/// Somehow, the difference among batch processing module and a real time processing module is
/// the statefulness. The first will keep the values and the buffers until consumption (stateful).
/// On the other hand, the second will calculate the value on a specific moment. The modules
/// don't even remember the time of the clock.
/// TODO: finish doc
pub trait Module {
    fn get_sample(&self, in_sample: f32, time: f32) -> f32 {
        self.behaviour(in_sample, time)
    }

    fn get_sample_w_aux(
        &mut self,
        in_sample: f32,
        time: f32,
        auxiliaries: HashMap<String, f32>,
    ) -> f32 {
        self.update_parameters(auxiliaries);
        self.behaviour(in_sample, time)
    }

    fn update_parameters(&mut self, auxiliaries: HashMap<String, f32>) {
        for (tag, value) in auxiliaries {
            let param = self.get_parameter_mutable(&tag);

            match param {
                Some(param) => param.set(value),
                None => {
                    error!("<b>Parameter tag <red>not found</><b>.</>")
                }
            }
        }
    }

    /// Fills the input buffer with new information. It may generate or modify the buffer.
    ///
    /// It also sets the clock forward and calls every function that needs to be updated on every
    /// tick, such as the [behavior](fn@Module::behaviour) or the update of the parameters.
    /// # Linker and generator modules
    /// A **generator module** does not use the incoming data, precisely because it is generating it.
    /// The input data is thus ignored, but the input buffer should be initialized.
    ///
    /// A **linker module** does modify the input buffer, so it does not ignore the incoming data.
    ///
    /// # Start the clock at a different time (partial batching)
    /// If you wanted to fill up the buffer until a specific moment, it is possible feeding the
    /// return value of this function into the [`fill_buffer_at`](fn@Module::fill_buffer_at)
    /// function.
    ///
    /// In the contrary, this function always starts with the clock at zero (the beginning).
    ///
    /// # Arguments
    /// * `buffer` - The buffer to fill/modify.
    /// * `auxiliaries` - A vector with the auxiliary inputs for the operation. Can be empty.
    ///
    /// # Returns
    /// The last value of the clock.
    // TODO stereo
    fn fill_buffer(&mut self, buffer: &mut Vec<f32>, auxiliaries: Vec<AuxiliaryInput>) -> f32 {
        self.fill_buffer_at(buffer, 0.0, auxiliaries)
    }

    /// Does the same as [`fill_buffer`](fn@Module::fill_buffer) function though a starting time
    /// for the clock can be specified.
    ///
    /// Please read [`fill_buffer`](fn@Module::fill_buffer) for more information.
    /// # Arguments
    /// * `buffer` - The buffer to fill/modify.
    /// * `auxiliaries` - A vector with the auxiliary inputs for the operation. Can be empty.
    /// * `start_at` - The value of the clock where it should start.
    ///
    /// # Returns
    /// The last value of the clock.
    fn fill_buffer_at(
        &mut self,
        buffer: &mut Vec<f32>,
        start_at: f32,
        mut auxiliaries: Vec<AuxiliaryInput>,
    ) -> f32 {
        #[cfg(feature = "verbose_modules")]
        {
            info!("<b>Running module <cyan>{}</>", self.get_name());
        }

        // TODO modularize this function not to reimplement the unnecessary.
        // maybe receive a closure with popping the values?
        warn!("<b>A <u>custom implementation</><b> for buffer filling with auxiliary inputs is recommended for better <yellow>performance</><b>.</>");

        let mut clock = Clock::new(44100); // TODO add get_sample_rate to Module trait
        clock.tick = start_at;

        #[cfg(feature = "verbose_modules")]
        {
            info!("<b>Auxiliary list: {} items</>", auxiliaries.len());
            for aux in auxiliaries.iter() {
                info!("  |_ <green>{} aux found</>", aux.get_tag());
            }
            println!();
        }

        // We reverse the auxiliary order because we are popping.
        // We want always the first element of the vector, nonetheless,
        // popping is faster than removing from the beginning.
        // As we don't need to access both ends of the vector, it is
        // easier just to reverse it.
        auxiliaries.reverse();

        // FILLING THE BUFFER IS THIS EASY
        buffer.iter_mut().for_each(|sample| {
            self.update_parameters(pop_auxiliaries(
                &mut auxiliaries,
                self.get_current_parameter_values(),
            ));
            *sample = self.get_sample(*sample, clock.inc())
        });

        clock.get_value()
    }

    /// Defines the behaviour of the module. Is it going to generate data? Is it going to clip the
    /// data under a threshold? Here is where the magic happens. The **behaviour is what defines
    /// a module.**
    /// # Arguments
    /// * `in_data`: the sample to modify, if any. Won't use it if creating a generator module.
    /// # Returns
    /// A generated or modified sample.
    fn behaviour(&self, in_data: f32, time: f32) -> f32;

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

    fn get_current_parameter_values(&self) -> HashMap<String, f32> {
        self.get_parameters()
            .iter()
            .map(|p| (p.get_tag().clone(), p.get_value()))
            .collect()
    }

    // USEFUL FOR DEBUGGING
    fn get_name(&self) -> String;
}

pub trait ModuleWrapper {
    fn gen_sample(&mut self, time: f32);
}

/// A **linker module** is a module able to consume data from modules, process it, and deliver it
/// to another module. An example of linker module would be an effect module or
/// an [ADSR](https://en.wikipedia.org/wiki/Envelope_(music)) module.
///
/// An example of **not** linker module would be an [generator module](struct@GeneratorModuleWrapper),
/// which does not use any input sample but instead generates one.
///
/// The [`LinkerModuleWrapper`](struct@LinkerModuleWrapper) does wrap a [Module] including
/// a [Consumer](https://docs.rs/ringbuf/latest/ringbuf/consumer/struct.Consumer.html) and a
/// [Producer](https://docs.rs/ringbuf/latest/ringbuf/producer/struct.Producer.html) of a
/// *ring buffer*. This allows the delivery of samples among modules in real time.
///
/// The *producer* of a linker module must be connected to the *consumer* of the **next module** in
/// the chain, and the *consumer* of the linker module must be connected to the *producer* of the
/// **previous module** in the chain.
pub struct LinkerModuleWrapper {
    module: Box<dyn Module>,
    consumer: ModuleConsumer,
    producer: ModuleProducer,
    aux_inputs: Vec<AuxiliaryInput>,
}

impl LinkerModuleWrapper {
    pub fn new(
        module: Box<dyn Module>,
        consumer: ModuleConsumer,
        producer: ModuleProducer,
        aux_inputs: Vec<AuxiliaryInput>,
    ) -> Self {
        Self {
            module,
            consumer,
            producer,
            aux_inputs,
        }
    }
}

impl ModuleWrapper for LinkerModuleWrapper {
    fn gen_sample(&mut self, time: f32) {
        if self.consumer.is_empty() {
            error!("<b>Buffer <red>empty</><b> in Linker Module.</>");
            error!("  |_ name: {}", self.module.get_name());
        } else {
            if self.producer.is_full() {
                error!("<b>Buffer <red>full</><b> in Linker Module.</>");
                error!("  |_ name: {}", self.module.get_name());
            } else {
                let prev = self.consumer.pop().unwrap();

                let aux_values = pop_auxiliaries(
                    &mut self.aux_inputs,
                    self.module.get_current_parameter_values(),
                );

                let value = self.module.get_sample_w_aux(prev, time, aux_values);

                self.producer.push(value).unwrap();
            }
        }
    }
}

/// A **generator module** is a module able to generate and deliver data to another module.
/// It should always be the first element of the chain. An example of generator module would be an
/// [Oscillator](struct@crate::Oscillator) module.
///
/// The [`GeneratorModuleWrapper`](struct@GeneratorModuleWrapper) does wrap a [Module] including
/// a [Producer](https://docs.rs/ringbuf/latest/ringbuf/producer/struct.Producer.html) of a
/// *ring buffer*. This allows the delivery of samples to another module in real time.
///
/// The *producer* of a generator module must be connected to the *consumer* of the **next module** in
/// the chain.
pub struct GeneratorModuleWrapper {
    module: Box<dyn Module>,
    producer: ModuleProducer,
    aux_inputs: Vec<AuxiliaryInput>,
}

impl GeneratorModuleWrapper {
    pub fn new(
        module: Box<dyn Module>,
        producer: ModuleProducer,
        aux_inputs: Vec<AuxiliaryInput>,
    ) -> Self {
        Self {
            module,
            producer,
            aux_inputs,
        }
    }
}

impl ModuleWrapper for GeneratorModuleWrapper {
    fn gen_sample(&mut self, time: f32) {
        if self.producer.is_full() {
            error!("<b>Buffer <red>full</><b> in Generator Module.</>");
            error!("  |_ name: {}", self.module.get_name());
        } else {
            let aux_values = pop_auxiliaries(
                &mut self.aux_inputs,
                self.module.get_current_parameter_values(),
            );

            let value = self.module.get_sample_w_aux(0.0, time, aux_values);

            self.producer.push(value).unwrap();
        }
    }
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
        if value <= self.max && value >= self.min {
            self.current = value;
        } else {
            #[cfg(feature = "verbose_modules")]
            {
                warn!("<b>Value <yellow>out of range</><b>.</>");
                warn!("  |_ Parameter: <yellow>{}</>", self.tag);
                warn!("  |_ Input value: <red>{}</>", value);
                warn!("  |_ Valid range: <green>[{}, {}]</>", self.min, self.max);
                warn!("  |_ Value kept back.");
                println!();
            }
        }
    }

    /// Increases the value of the parameter upon maximum.
    pub fn inc(&mut self) {
        // if value exceeds the maximum, keep the max value.
        if self.current + self.step > self.max {
            self.current = self.max;
            warn!("<b>Trying to <yellow>exceed</> <b>the value over the maximum.</>");

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
            warn!("<b>Trying to <yellow>exceed</> <b>the value under the minimum.</>");

        // otherwise, keep lowering it
        } else {
            self.current -= self.step;
        }
    }
}

/// A builder pattern to create parameters in a modular fashion. Check [Parameter] for all the
/// information about the fields and how should it be used.
/// # Example
/// ```rust
/// // A parameter for changing the value of the frequency at any given moment.
/// ParameterBuilder::new("frequency".to_string())
///     .with_min(22000.0) // 22k Hz for the max
///     .with_min(10.0) // 10  Hz for the min
///     .with_step(10.0) // Increments from 10 to 10 Hz
///     .with_default(440.0) // The value starting at is 440 Hz
///     .build()
///     .unwrap();
/// ```
pub struct ParameterBuilder {
    /// Maximum value. Defaults on 1.0
    max: Option<f32>,
    /// Minimum value. Defaults on 0.0
    min: Option<f32>,
    /// Step value. Defaults on 0.1
    step: Option<f32>,
    /// Default value. Defaults on 0.0
    default: Option<f32>,
    /// Tag (name) of the filed. Serves as identifier and should not be duplicated.
    tag: String,
}

impl ParameterBuilder {
    /// Creates a new builder with all values set at default.
    ///
    /// **Requires** the tag of the parameter, which servers as **identifier**.
    pub fn new(tag: String) -> Self {
        Self {
            max: None,
            min: None,
            step: None,
            default: None,
            tag,
        }
    }

    /// Sets the maximum value of the [Parameter].
    pub fn with_max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the mimimum value of the [Parameter].
    pub fn with_min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the step of the [Parameter].
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets the default value of the [Parameter].
    pub fn with_default(mut self, default: f32) -> Self {
        self.default = Some(default);
        self
    }

    /// Generates a [Parameter] from the specified values. Performs some integrity checks.
    pub fn build(self) -> Result<Parameter, String> {
        let max = self.max.unwrap_or(1.0);
        let min = self.min.unwrap_or(0.0);
        let step = self.step.unwrap_or(0.1);
        let default = self.default.unwrap_or(0.0);
        let current = default.clone();
        let tag = self.tag;

        if max < min {
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

/// An **Auxiliary Input** allows routing the output of a module to another one. They can also be
/// understood as **side chain connections**.
/// # Parameters
/// Auxiliary inputs make no sense if they are not linked with a [Parameter]. Auxiliary Inputs are
/// owned by the same module to be modulated and, in order to work, the tags of both Auxiliary
/// Input and Parameters must match.
///
/// # Value translation (important!)
/// On top of that, Aux Inputs perform a [translation](fn@AuxiliaryInput::translate) of the incoming data. Although *every module*
/// outputs values from -1 to 1 (f32 sample format) which is perfect for raw output data,
/// these values don't fit the majority of modules (no to say none) and, thus, the values need to
/// be adjusted. This is important to bear in mind as when defining a Auxiliary Input no to get
/// errors from invalid data inputs.
pub struct AuxiliaryInput {
    /// [Parameter] to which the Auxiliary Input shall be linked with (must match with the tag field of the parameter in order to work).
    tag: String,
    /// The buffer with the output of the **modulator** module.
    data: AuxDataHolder,
    /// The *maximum* value of the **input** of the parameter. Don't need to match with the max of
    /// the associated parameter, but must be lower or equal to work properly.
    max: f32,
    /// The *minimum* value of the **input** of the parameter. Don't need to match with the min of
    /// the associated parameter, but must be greater or equal to work properly.
    min: f32,
}

impl AuxiliaryInput {
    /// Retrieves the tag of the Auxiliary Input.
    pub fn get_tag(&self) -> String {
        self.tag.to_string()
    }

    /// Pops the latest value of the auxiliary input. Additionally, it performs a translation
    /// from the values ranging from -1 to 1 that every module should output into the max and
    /// min values specified when built.
    fn pop(&mut self) -> Option<f32> {
        match &mut (self.data) {
            AuxDataHolder::Batch(ref mut buffer) => match buffer.pop() {
                Some(x) => Some(self.translate(x)),
                None => None,
            },
            AuxDataHolder::RealTime(ref mut consumer) => match consumer.pop() {
                Some(x) => Some(self.translate(x)),
                None => None,
            },
        }
    }

    /// Translation of the values from [-1, 1] to [min, max]. Read the [AuxiliaryInput] description
    /// for a full explanation.
    ///
    /// When getting a value from a AuxiliaryInput a previous translation will be performed to
    /// match the output of any module [-1, 1] to the values set when
    /// [building](struct@AuxInputBuilder) the auxiliary.
    /// ```rust
    /// let buffer = vec![0.0f32; 10];
    ///         
    /// AuxInputBuilder::new("amplitude", buffer)
    ///     .with_min(0.0)
    ///     .with_max(1.0)
    ///     .build()
    ///     .unwrap();
    ///         
    /// // Input: -1.0; Output: 0.0
    /// // Input:  0.0; Output: 0.5
    /// // Input:  1.0; Output: 1.0
    /// ```
    fn translate(&self, value: f32) -> f32 {
        let size = self.max - self.min;

        ((size * value + size) / 2.0) + self.min
    }
}

/// The proper way of creating an Auxiliary Input. To understand how they work and should
/// be used, please check the **[AuxiliaryInput]** page.
/// # Usage
/// ```rust
/// let buffer = vec![0.0f32; 10]; // Buffer with the output of the previous module
///
/// // Linking the auxiliary with the frequency (FM)
/// let mut aux = AuxInputBuilder::new("frequency", buffer).build().unwrap();
///
/// // Creating a buffer and an oscillator
/// let mut buffer = vec![0.0f32; 10];
/// let mut osc = OscillatorBuilder::new().build().unwrap();
///
/// // Fill the data of the buffer with the new auxiliary.
/// osc.fill_buffer_w_aux(&mut buffer, Some(vec![&mut aux]));
///
/// ```
pub struct AuxInputBuilder {
    /// Tag matching the [Parameter] field.
    tag: String,
    /// Buffer to hold the data of the previous module.
    data: AuxDataHolder,
    /// Maximum value. Defaults on 1.0
    max: Option<f32>,
    /// Minimum value. Defaults on 0.0
    min: Option<f32>,
}

impl AuxInputBuilder {
    /// Creates a new [AuxiliaryInput] builder with all values at default.
    /// Requires a tag and a buffer.
    /// # Arguments
    /// * `tag` - Name of the parameter linked with the Aux Input.
    /// * `buffer` - Buffer containing the data generated by the previous module (modulator).
    pub fn new(tag: &str, data: AuxDataHolder) -> Self {
        Self {
            tag: tag.to_string(),
            data,
            max: None,
            min: None,
        }
    }

    /// Defines a max value for the [AuxiliaryInput].
    pub fn with_max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    /// Defines a min value for the [AuxiliaryInput].
    pub fn with_min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_all_yaml(mut self, max: Option<f32>, min: Option<f32>) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Generates an [AuxiliaryInput] from the values specified.
    pub fn build(self) -> Result<AuxiliaryInput, String> {
        let max = self.max.unwrap_or(1.0);
        let min = self.min.unwrap_or(0.0);

        if max < min {
            return Err("Invalid range".to_string());
        }

        Ok(AuxiliaryInput {
            tag: self.tag,
            data: self.data,
            max,
            min,
        })
    }
}

pub enum AuxDataHolder {
    Batch(Vec<f32>),
    RealTime(ModuleConsumer),
}

impl AuxDataHolder {
    pub fn is_batch(&self) -> bool {
        matches!(*self, Self::Batch(_))
    }

    pub fn is_real_time(&self) -> bool {
        matches!(*self, Self::RealTime(_))
    }

    pub fn get_buffer(&self) -> Option<&Vec<f32>> {
        match self {
            Self::Batch(buffer) => Some(buffer),
            _ => None,
        }
    }

    pub fn get_consumer(&self) -> Option<&ModuleConsumer> {
        match self {
            Self::RealTime(consumer) => Some(&consumer),
            _ => None,
        }
    }
}

/// A structure with some bundled methods to easily manage time synchronization.
pub struct Clock {
    tick: f32,
    sample_rate: f32,
}

impl Clock {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            tick: 0.0,
            sample_rate: sample_rate as f32,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.tick
    }

    pub fn post_inc(&mut self) -> f32 {
        self.tick = (self.tick + 1.0) % self.sample_rate;
        self.tick
    }

    pub fn inc(&mut self) -> f32 {
        let prev = self.tick;
        self.tick = (self.tick + 1.0) % self.sample_rate;
        prev
    }
}

#[cfg(test)]
mod test {
    use super::AuxDataHolder::Batch;
    use super::{AuxInputBuilder, AuxiliaryInput};
    use super::{Parameter, ParameterBuilder};

    mod parameter_builder_tests {
        use super::*;
        use log::info;
        use simplelog::__private::paris::Logger;

        fn get_logger() -> Logger<'static> {
            Logger::new()
        }

        #[test]
        fn test_default() {
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

    mod parameter_tests {
        use super::*;
        fn get_parameter() -> Parameter {
            ParameterBuilder::new("test".to_string())
                .with_max(1.2)
                .with_min(0.1)
                .with_default(0.5)
                .with_step(0.2)
                .build()
                .unwrap()
        }

        #[test]
        fn test_get_tag() {
            let parameter = get_parameter();

            assert_eq!(parameter.get_tag(), "test");
        }

        #[test]
        fn test_get_value() {
            let mut parameter = get_parameter();

            assert_eq!(parameter.get_value(), 0.5, "Current value mismatch");
            parameter.set(0.1);
            assert_eq!(parameter.get_value(), 0.1, "Current value mismatch");
        }

        #[test]
        fn test_set_value() {
            let mut parameter = get_parameter();

            parameter.set(1.2);
            assert_eq!(parameter.get_value(), 1.2, "Current value mismatch");
            parameter.set(-1.0);
            assert_eq!(parameter.get_value(), 1.2, "Smaller than check wrong");
            parameter.set(10.0);
            assert_eq!(parameter.get_value(), 1.2, "Greater than check wrong");
        }

        #[test]
        fn test_inc() {
            let mut parameter = get_parameter();

            assert_eq!(parameter.get_value(), 0.5, "Default item is different");
            parameter.inc();
            assert_eq!(parameter.get_value(), 0.7, "Decrease not working");

            parameter.set(1.1);
            parameter.inc();
            assert_eq!(
                parameter.get_value(),
                parameter.max,
                "Increase out of bounds"
            )
        }

        #[test]
        fn test_dec() {
            let mut parameter = get_parameter();

            assert_eq!(parameter.get_value(), 0.5, "Default item is different");
            parameter.dec();
            assert_eq!(parameter.get_value(), 0.3, "Decrease not working");

            parameter.set(0.2);
            parameter.dec();
            assert_eq!(
                parameter.get_value(),
                parameter.min,
                "Decrease out of bounds"
            )
        }
    }

    mod auxiliary_input_builder_tests {
        use super::*;

        #[test]
        fn test_default() {
            let aux = AuxInputBuilder::new("test", Batch(vec![0.0]))
                .build()
                .unwrap();
            assert_eq!(aux.tag, "test", "Default tag mismatch");
            assert_eq!(
                *aux.data.get_buffer().unwrap(),
                vec![0.0],
                "Default buffer mismatch"
            );
            assert_eq!(aux.max, 1.0, "Default max mismatch");
            assert_eq!(aux.min, 0.0, "Default min mismatch");
        }

        #[test]
        fn test_with_all() {
            let aux = AuxInputBuilder::new("test", Batch(vec![0.0]))
                .with_max(10.0)
                .with_min(5.0)
                .build()
                .unwrap();

            assert_eq!(aux.tag, "test", "Default tag mismatch");
            assert_eq!(
                *aux.data.get_buffer().unwrap(),
                vec![0.0],
                "Default buffer mismatch"
            );
            assert_eq!(aux.max, 10.0, "Test max mismatch");
            assert_eq!(aux.min, 5.0, "Default min mismatch");
        }

        #[test]
        #[should_panic]
        fn test_with_invalid_range() {
            AuxInputBuilder::new("test", Batch(vec![0.0]))
                .with_max(0.0)
                .with_min(1.0)
                .build()
                .unwrap();
        }
    }

    mod auxiliary_input_test {
        use super::*;
        use crate::module::AuxDataHolder::RealTime;
        use crate::module::{ModuleConsumer, ModuleProducer};
        use ringbuf::HeapRb;

        fn get_aux() -> AuxiliaryInput {
            let buffer: Vec<f32> = vec![-1.0, 0.0, 1.0];
            AuxInputBuilder::new("test", Batch(buffer))
                .with_max(20.0)
                .with_min(10.0)
                .build()
                .unwrap()
        }

        fn get_aux_rt(cons: ModuleConsumer) -> AuxiliaryInput {
            AuxInputBuilder::new("test", RealTime(cons))
                .with_max(20.0)
                .with_min(10.0)
                .build()
                .unwrap()
        }

        #[test]
        fn test_get_tag() {
            let aux = get_aux();

            assert_eq!(aux.get_tag(), "test", "Default name mismatch");
        }

        #[test]
        fn test_pop_aux() {
            let mut aux = get_aux();

            assert_eq!(aux.pop(), Some(aux.max));
            assert_eq!(aux.pop(), Some(15.0));
            assert_eq!(aux.pop(), Some(aux.min));
            assert_eq!(aux.pop(), None);
        }

        #[test]
        fn test_pop_rt_aux() {
            let rb: HeapRb<f32> = HeapRb::new(4);
            let (mut p, c) = rb.split();
            let mut rt_aux = get_aux_rt(c);

            p.push(1.0).unwrap();
            p.push(0.0).unwrap();
            p.push(-1.0).unwrap();

            assert_eq!(rt_aux.pop().unwrap(), rt_aux.max);
            assert_eq!(rt_aux.pop().unwrap(), 15.0);
            assert_eq!(rt_aux.pop().unwrap(), rt_aux.min);
        }

        #[test]
        fn test_translation() {
            let buffer: Vec<f32> = vec![-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];
            let mut aux = AuxInputBuilder::new("test", Batch(buffer))
                .with_min(-10.0)
                .with_max(10.0)
                .build()
                .unwrap();

            assert_eq!(aux.pop(), Some(aux.max));
            assert_eq!(aux.pop(), Some(7.5));
            assert_eq!(aux.pop(), Some(5.0));
            assert_eq!(aux.pop(), Some(2.5));
            assert_eq!(aux.pop(), Some(0.0));
            assert_eq!(aux.pop(), Some(-2.5));
            assert_eq!(aux.pop(), Some(-5.0));
            assert_eq!(aux.pop(), Some(-7.5));
            assert_eq!(aux.pop(), Some(aux.min));
        }
    }

    mod aux_data_holder_test {}
}
