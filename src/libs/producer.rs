use tokio::sync::{mpsc, watch};

pub trait Producer<Input> {
    fn matches(&self, input_event: &Input) -> bool;
    fn start(&self, sender: mpsc::UnboundedSender<Input>, shutdown: watch::Receiver<bool>);
}
