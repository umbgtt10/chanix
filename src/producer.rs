use crossbeam::channel::{Receiver, Sender};

pub trait Producer<Input> {
    fn matches(&self, input_event: &Input) -> bool;
    fn start(&self, sender: Sender<Input>, shutdown: Receiver<()>);
}
