use crate::module::ModuleConsumer;

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
    pub fn pop(&mut self) -> Option<f32> {
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

    pub fn get_max(&self) -> f32 {
        self.max
    }

    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get_data(&self) -> &AuxDataHolder {
        &self.data
    }

    pub fn get_mut_data(&mut self) -> &mut AuxDataHolder {
        &mut self.data
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
        let from_range = (-1.0, 1.0);

        // ( (old_value - old_min) / (old_max - old_min) ) * (new_max - new_min) + new_min
        ((value - from_range.0) / (from_range.1 - from_range.0)) * (self.max - self.min) + self.min
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

    pub fn reverse_buffer(&mut self) -> Result<(), String> {
        match self {
            Self::Batch(buffer) => {
                buffer.reverse();
                Ok(())
            }
            _ => Err(String::from(
                "Buffer cannot be reversed in real time processing",
            )),
        }
    }

    pub fn get_consumer(&self) -> Option<&ModuleConsumer> {
        match self {
            Self::RealTime(consumer) => Some(&consumer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod auxiliary_input_builder_tests {
        use super::*;
        use crate::module::AuxDataHolder::Batch;

        #[test]
        fn test_default() {
            let aux = AuxInputBuilder::new("test", Batch(vec![0.0]))
                .build()
                .unwrap();
            assert_eq!(aux.get_tag(), "test", "Default tag mismatch");
            assert_eq!(
                *aux.get_data().get_buffer().unwrap(),
                vec![0.0],
                "Default buffer mismatch"
            );
            assert_eq!(aux.get_max(), 1.0, "Default max mismatch");
            assert_eq!(aux.get_min(), 0.0, "Default min mismatch");
        }

        #[test]
        fn test_with_all() {
            let aux = AuxInputBuilder::new("test", Batch(vec![0.0]))
                .with_max(10.0)
                .with_min(5.0)
                .build()
                .unwrap();

            assert_eq!(aux.get_tag(), "test", "Default tag mismatch");
            assert_eq!(
                *aux.get_data().get_buffer().unwrap(),
                vec![0.0],
                "Default buffer mismatch"
            );
            assert_eq!(aux.get_max(), 10.0, "Test max mismatch");
            assert_eq!(aux.get_min(), 5.0, "Default min mismatch");
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
        use crate::module::AuxDataHolder::{Batch, RealTime};
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

            assert_eq!(aux.pop(), Some(aux.get_max()));
            assert_eq!(aux.pop(), Some(15.0));
            assert_eq!(aux.pop(), Some(aux.get_min()));
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

            assert_eq!(rt_aux.pop().unwrap(), rt_aux.get_max());
            assert_eq!(rt_aux.pop().unwrap(), 15.0);
            assert_eq!(rt_aux.pop().unwrap(), rt_aux.get_min());
        }

        #[test]
        fn test_translation() {
            let buffer: Vec<f32> = vec![-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];
            let mut aux = AuxInputBuilder::new("test", Batch(buffer))
                .with_min(-10.0)
                .with_max(10.0)
                .build()
                .unwrap();

            assert_eq!(aux.pop(), Some(aux.get_max()));
            assert_eq!(aux.pop(), Some(7.5));
            assert_eq!(aux.pop(), Some(5.0));
            assert_eq!(aux.pop(), Some(2.5));
            assert_eq!(aux.pop(), Some(0.0));
            assert_eq!(aux.pop(), Some(-2.5));
            assert_eq!(aux.pop(), Some(-5.0));
            assert_eq!(aux.pop(), Some(-7.5));
            assert_eq!(aux.pop(), Some(aux.get_min()));
        }
    }

    mod aux_data_holder_test {}
}
