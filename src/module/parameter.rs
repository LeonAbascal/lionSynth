use simplelog::{error, warn};

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

#[cfg(test)]
mod tests {
    use super::*;
    mod parameter_builder_tests {
        use super::*;
        use simplelog::__private::paris::Logger;
        use simplelog::info;

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
}
