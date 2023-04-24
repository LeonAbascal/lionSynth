mod oscillator;
mod sum;

pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorBuilder};
pub use crate::bundled_modules::sum::{Sum2In, Sum2InBuilder, VarSum, VarSumBuilder};

pub mod prelude {
    pub use crate::bundled_modules::oscillator::{Oscillator, OscillatorBuilder};
    pub use crate::bundled_modules::sum::{Sum2In, Sum2InBuilder, VarSum, VarSumBuilder};
}

mod debug_modules;
mod oscillator_math;

pub mod debug {
    pub use crate::bundled_modules::debug_modules::{OscDebug, PassTrough};
}
