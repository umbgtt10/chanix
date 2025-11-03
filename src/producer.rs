use crossbeam::channel::Sender;
use std::sync::{Arc, atomic::AtomicBool};

pub trait Producer<Input> {
    fn matches(&self, input_event: &Input) -> bool;
    fn start(&self, sender: Sender<Input>, shutdown: Arc<AtomicBool>);
}
