use crate::infra::input_events::EventType3;
use crate::infra::input_events::InputEvents;
use chanix::producer::Producer;
use crossbeam::channel::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

#[derive(Clone)]
pub struct Producer3 {
    name: String,
    start_id: u32,
    end_id: u32,
}

impl Producer3 {
    pub fn new(name: &str, start_id: u32, end_id: u32) -> Self {
        Producer3 {
            name: name.to_string(),
            start_id,
            end_id,
        }
    }
}

impl Producer<InputEvents> for Producer3 {
    fn matches(&self, input_event: &InputEvents) -> bool {
        matches!(input_event, InputEvents::EventType3(_))
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

                let message = InputEvents::EventType3(EventType3::new(id, id));

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
