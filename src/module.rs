/// A **linker module** is a module able to connect into other. An example of linker module
/// would be an effect module or an [ADSR](https://en.wikipedia.org/wiki/Envelope_(music)) module. An example of not linker module would
/// be an generator module, which does not use any input sample but
pub struct LinkerModule {
    // TODO
}

/// A module is the most basic unit of the modular synthesizer.
///
pub trait Module {
    // fn get_sample(&self); // real time behaviour?

    /// Fills the input buffer with new information. It may generate or modify the buffer.
    /// Comes with a default implementation which automagically increases the clock of the
    /// module and

    // TODO stereo
    fn fill_buffer(&mut self, buffer: &mut Vec<f32>) {
        let mut count = 0;
        for item in buffer {
            count = (count + 1) % 10;
            self.tick();
            *item = self.behaviour(*item);

            #[cfg(feature = "verbose_modules")]
            {
                println!("[ {} ] {}", count, item);
            }
        }
    }

    /// Defines the behaviour of the module. Is it going to generate data? To clip the data under
    /// a threshold? Here is where the magic happens. The behaviour is what defines a module.
    /// # Arguments
    /// * `in_data`: the sample to modify, if any. Won't use it if creating a generator module.
    /// # Returns
    /// A generated or modified sample
    fn behaviour(&self, in_data: f32) -> f32;

    /// Will define how the clock goes forward. Useful for timed operations
    fn tick(&mut self); // TODO: consider making the tick common with an associated function (class method) or a sync-er structure in the main flow
    fn get_clock(&self) -> f32; // TODO: remove? tick already enforces the clock in the struct
}
