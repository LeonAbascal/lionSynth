use crate::module::module::pop_auxiliaries;
use crate::module::*;
use simplelog::warn;
use std::collections::LinkedList;

pub trait ModuleWrapper {
    fn gen_sample(&mut self, time: f32);
    fn get_name(&self) -> String;
    fn get_producer(&self) -> &ModuleProducer;
}

/// A **linker module** is a module able to consume data from modules, process it, and deliver it
/// to another module. An example of linker module would be an effect module or
/// an [ADSR](https://en.wikipedia.org/wiki/Envelope_(music)) module.
///
/// An example of **not** linker module would be an [generator module](struct@GeneratorModuleWrapper),
/// which does not use any input sample but instead generates one.
///
/// The [`LinkerModuleWrapper`](struct@LinkerModuleWrapper) does wrap a [Module] including
/// a [Consumer](https://docs.rs/ringbuf/latest/ringbuf/consumer/struct.Consumer.html) and a
/// [Producer](https://docs.rs/ringbuf/latest/ringbuf/producer/struct.Producer.html) of a
/// *ring buffer*. This allows the delivery of samples among modules in real time.
///
/// The *producer* of a linker module must be connected to the *consumer* of the **next module** in
/// the chain, and the *consumer* of the linker module must be connected to the *producer* of the
/// **previous module** in the chain.
pub struct LinkerModuleWrapper {
    module: Box<dyn Module>,
    consumer: ModuleConsumer,
    producer: ModuleProducer,
    aux_inputs: Vec<AuxiliaryInput>,
}

impl LinkerModuleWrapper {
    pub fn new(
        module: Box<dyn Module>,
        consumer: ModuleConsumer,
        producer: ModuleProducer,
        aux_inputs: Vec<AuxiliaryInput>,
    ) -> Self {
        Self {
            module,
            consumer,
            producer,
            aux_inputs,
        }
    }
}

impl ModuleWrapper for LinkerModuleWrapper {
    fn gen_sample(&mut self, time: f32) {
        if self.consumer.is_empty() {
            warn!("<b>Buffer <yellow>empty</><b> in Linker Module.</>");
            warn!("  |_ name: {}", self.module.get_name());
        } else {
            if self.producer.is_full() {
                warn!("<b>Buffer <yellow>full</><b> in Linker Module.</>");
                warn!("  |_ name: {}", self.module.get_name());
            } else {
                let prev = self.consumer.pop().unwrap();

                let aux_values = pop_auxiliaries(
                    &mut self.aux_inputs,
                    self.module.get_current_parameter_values(),
                );

                let value = self.module.get_sample_w_aux(prev, time, aux_values);

                self.producer.push(value).unwrap();
            }
        }
    }

    fn get_name(&self) -> String {
        self.module.get_name().clone()
    }

    fn get_producer(&self) -> &ModuleProducer {
        &self.producer
    }
}

/// A **generator module** is a module able to generate and deliver data to another module.
/// It should always be the first element of the chain. An example of generator module would be an
/// [Oscillator](struct@crate::bundled_modules::Oscillator) module.
///
/// The [`GeneratorModuleWrapper`](struct@GeneratorModuleWrapper) does wrap a [Module] including
/// a [Producer](https://docs.rs/ringbuf/latest/ringbuf/producer/struct.Producer.html) of a
/// *ring buffer*. This allows the delivery of samples to another module in real time.
///
/// The *producer* of a generator module must be connected to the *consumer* of the **next module** in
/// the chain.
pub struct GeneratorModuleWrapper {
    module: Box<dyn Module>,
    producer: ModuleProducer,
    aux_inputs: Vec<AuxiliaryInput>,
}

impl GeneratorModuleWrapper {
    pub fn new(
        module: Box<dyn Module>,
        producer: ModuleProducer,
        aux_inputs: Vec<AuxiliaryInput>,
    ) -> Self {
        Self {
            module,
            producer,
            aux_inputs,
        }
    }
}

impl ModuleWrapper for GeneratorModuleWrapper {
    fn gen_sample(&mut self, time: f32) {
        if self.producer.is_full() {
            warn!("<b>Buffer <yellow>full</><b> in Generator Module.</>");
            warn!("  |_ name: {}", self.module.get_name());
        } else {
            let aux_values = pop_auxiliaries(
                &mut self.aux_inputs,
                self.module.get_current_parameter_values(),
            );

            let value = self.module.get_sample_w_aux(0.0, time, aux_values);

            self.producer.push(value).unwrap();
        }
    }

    fn get_name(&self) -> String {
        self.module.get_name().clone()
    }

    fn get_producer(&self) -> &ModuleProducer {
        &self.producer
    }
}

/// A structure with some bundled methods to easily manage time synchronization.
pub struct Clock {
    tick: f32,
    sample_rate: f32,
}

impl Clock {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            tick: 0.0,
            sample_rate: sample_rate as f32,
        }
    }

    pub fn new_at(sample_rate: i32, start_at: f32) -> Self {
        Self {
            tick: start_at,
            sample_rate: sample_rate as f32,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.tick
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn post_inc(&mut self) -> f32 {
        self.tick = (self.tick + 1.0) % self.sample_rate;
        self.tick
    }

    pub fn inc(&mut self) -> f32 {
        let prev = self.tick;
        self.tick = (self.tick + 1.0) % self.sample_rate;
        prev
    }
}

pub struct CoordinatorEntity {
    clock: Clock,
    wrapper_chain: LinkedList<Box<dyn ModuleWrapper>>,
}

impl CoordinatorEntity {
    pub fn new(sample_rate: i32, chain: LinkedList<Box<dyn ModuleWrapper>>) -> Self {
        Self {
            clock: Clock::new(sample_rate),
            wrapper_chain: chain,
        }
    }

    pub fn get_mut_wrapper_chain(&mut self) -> &mut LinkedList<Box<dyn ModuleWrapper>> {
        &mut self.wrapper_chain
    }

    pub fn tick(&mut self) {
        self.wrapper_chain.iter_mut().for_each(|module| {
            module.gen_sample(self.clock.get_value());
        });

        // POST OPERATIONS
        self.clock.inc();
    }
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled_modules::debug::PassTrough;
    use crate::bundled_modules::OscillatorBuilder;
    use ringbuf::HeapRb;
    use std::thread;

    #[test]
    fn test_wrappers() {
        let mut wrapper_chain: Vec<Box<dyn ModuleWrapper>> = Vec::new();

        let mut osc = OscillatorBuilder::new().build().unwrap();
        let mut test_osc = OscillatorBuilder::new().build().unwrap();
        let mut pt = PassTrough::new();

        let rb1: HeapRb<f32> = HeapRb::new(10);
        let rb2: HeapRb<f32> = HeapRb::new(10);
        let (p1, c1) = rb1.split();
        let (p2, mut c2) = rb2.split();

        let mut w1 = GeneratorModuleWrapper::new(Box::new(osc), p1, vec![]);
        let mut w2 = LinkerModuleWrapper::new(Box::new(pt), c1, p2, vec![]);

        let time = 0.0;
        w1.gen_sample(time);
        w2.gen_sample(time);

        let post_chain = c2.pop().unwrap();
        assert_eq!(test_osc.get_sample(0.0, time), post_chain);

        for time in 0..44100 {
            let time = time as f32;
            w1.gen_sample(time);
            w2.gen_sample(time);

            assert_eq!(test_osc.get_sample(0.0, time), c2.pop().unwrap());
        }

        for time in 0..44100 {
            let time = time as f32;
            w1.gen_sample(time);
            w2.gen_sample(time);
        }

        let handle = thread::spawn(move || {
            for time in 0..44100 {
                if !c2.is_empty() {
                    assert_eq!(test_osc.get_sample(0.0, time as f32), c2.pop().unwrap());
                }
            }
        });

        handle.join().unwrap();
    }
}
