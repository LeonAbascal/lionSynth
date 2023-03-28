mod oscillator;
pub use crate::bundled_modules::oscillator::Oscillator;

pub mod prelude {
    pub use crate::bundled_modules::oscillator::Oscillator;
}

mod debug_modules;
mod oscillator_math;

pub mod debug {
    pub use crate::bundled_modules::debug_modules::{OscDebug, PassTrough};
}
