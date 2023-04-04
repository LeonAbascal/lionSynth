mod oscillator;
pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorBuilder};

pub mod prelude {
    pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorBuilder};
}

mod debug_modules;
mod oscillator_math;

pub mod debug {
    pub use crate::bundled_modules::debug_modules::{OscDebug, PassTrough};
}
