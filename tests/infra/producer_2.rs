use crate::infra::input_events::EventType2;
use crate::infra::input_events::InputEvents;
use chanix::producer::Producer;
use crossbeam::channel::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

#[derive(Clone)]
pub struct Producer2 {
    name: String,
    start_id: u32,
    end_id: u32,
}

impl Producer2 {
    pub fn new(name: &str, start_id: u32, end_id: u32) -> Self {
        Producer2 {
            name: name.to_string(),
            start_id,
            end_id,
        }
    }
}

impl Producer<InputEvents> for Producer2 {
    fn matches(&self, input_event: &InputEvents) -> bool {
        matches!(input_event, InputEvents::EventType2(_))
    }

    fn start(&self, sender: Sender<InputEvents>, shutdown: Arc<AtomicBool>) {
        let name = self.name.clone();
        let start_id = self.start_id;
        let end_id = self.end_id;

        thread::spawn(move || {
            println!("[Producer: {name}] Started.");

            for id in start_id..=end_id {
                if shutdown.load(Ordering::SeqCst) {
                    println!("[Producer: {name}] Shutting down.");
                    break;
                }

                let message = InputEvents::EventType2(EventType2::new(id, id as f64));

                println!("[Producer: {name}] Sending: {:?}", message);
                if sender.send(message).is_err() {
                    println!("[Producer: {name}] Failed to send a message. The channel is closed.");
                    break;
                }
            }

            println!("[Producer: {name}] Finished.");
        });
    }
}
