mod oscillator;
pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorFactory};

pub mod prelude {
    pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorFactory};
}

mod debug_modules;
mod oscillator_math;

pub mod debug {
    pub use crate::bundled_modules::debug_modules::{OscDebug, PassTrough};
}
