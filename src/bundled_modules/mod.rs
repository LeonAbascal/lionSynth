mod osc;
mod sum;

pub use crate::bundled_modules::osc::{Oscillator, OscillatorBuilder};
pub use crate::bundled_modules::sum::{Sum2In, Sum2InBuilder, VarSum, VarSumBuilder};

pub mod prelude {
    pub use crate::bundled_modules::osc::{Oscillator, OscillatorBuilder};
    pub use crate::bundled_modules::sum::{
        Sum2In, Sum2InBuilder, Sum3In, Sum3InBuilder, VarSum, VarSumBuilder,
    };
}

mod debug_modules;

pub mod debug {
    pub use crate::bundled_modules::debug_modules::{OscDebug, PassTrough};
}

/// These constants are used for readability and avoid duplication.
/// The only one I would dare changing is the [`OVER_GAIN`](const@super::consts::OVER_GAIN).
pub mod consts {
    /// Maximum gain level. Used to compensate low levels.
    pub(crate) const OVER_GAIN: f32 = 2.0;
    /// Minimum gain level. Do you want to get lower than zero?
    pub(crate) const MIN_GAIN: f32 = 0.0;
    /// For the maximum value of a signal in f32 format (1.0)
    pub(crate) const AUDIO_RANGE_TOP: f32 = 1.0;
    /// For the minimum value of a signal in f32 format (-1.0)
    pub(crate) const AUDIO_RANGE_BOT: f32 = -1.0;
}
