use simplelog::{error, info, warn};
use std::collections::HashMap;

use super::*;

/// Receives a list of the last values of the given auxiliaries.
pub fn pop_auxiliaries(
    auxiliaries: &mut Vec<AuxiliaryInput>,
    current_values: HashMap<String, f32>,
) -> HashMap<String, f32> {
    let result: HashMap<String, f32>;

    result = auxiliaries
        .iter_mut()
        .map(|aux| {
            let tag = aux.get_tag().clone(); // Gets the parameter tag is associated with

            let value = match aux.pop() {
                // Gets the next sample in the vector
                Some(value) => value,
                None => {
                    let prev_value = *current_values.get(&tag).unwrap();
                    warn!("<b>Values of auxiliary list <yellow>exhausted</><b>. It is perfectly normal for the first samples of the chain.</>");
                    warn!("Defaulting to previous value: {}", prev_value);
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
                    error!("<b>Parameter tag <red>not found</><b>.</>");
                    error!("  |_ name: {}", tag);
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

        let mut clock = Clock::new_at(44100, start_at); // TODO add get_sample_rate to Module trait

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
        auxiliaries
            .iter_mut()
            .for_each(|aux| aux.get_mut_data().reverse_buffer().unwrap());

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

    /*/// Adds a parameter to the list of parameters. If the tag is already in the list,
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
    }*/

    /// Retrieves a **mutable** parameter given its tag, if exists.
    fn get_parameter_mutable(&mut self, tag: &str) -> Option<&mut Parameter> {
        match self.get_parameters_mutable() {
            Some(parameters) => parameters.into_iter().find(|p| p.get_tag() == tag),
            None => None,
        }
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
        match self.get_parameters() {
            Some(parameters) => parameters.into_iter().find(|p| p.get_tag() == tag),
            None => None,
        }
    }

    /// Gets all parameters in the list. Used to enforce the presence of a parameter
    /// list in every module struct.
    fn get_parameters(&self) -> Option<Vec<&Parameter>>;

    /// Gets all mutable parameters in the list.
    fn get_parameters_mutable(&mut self) -> Option<Vec<&mut Parameter>>;

    fn get_parameter_count(&self) -> usize {
        match self.get_parameters() {
            Some(parameters) => parameters.len(),
            None => 0,
        }
    }

    fn get_current_parameter_values(&self) -> HashMap<String, f32> {
        match self.get_parameters() {
            Some(parameters) => parameters
                .into_iter()
                .map(|p| {
                    let tag = p.get_tag().clone();
                    let value = p.get_value();
                    (tag, value)
                })
                .collect(),
            None => HashMap::new(),
        }
    }

    // USEFUL FOR DEBUGGING
    fn get_name(&self) -> String;
}

#[cfg(test)]
mod test {}
