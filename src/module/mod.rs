mod aux_input;
mod module;
mod parameter;
mod real_time;

pub use aux_input::{AuxDataHolder, AuxInputBuilder, AuxiliaryInput};
pub use module::Module;
pub use parameter::{Parameter, ParameterBuilder};
pub use real_time::{
    Clock, CoordinatorEntity, GeneratorModuleWrapper, LinkerModuleWrapper, ModuleWrapper,
};

// TYPES
use ringbuf::{Consumer, Producer, SharedRb};
use std::mem::MaybeUninit;
use std::sync::Arc;

/// Alias for ring buffer consumer used in modules
pub type ModuleConsumer = Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
/// Alias for ring buffer producer used in modules
pub type ModuleProducer = Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>;
