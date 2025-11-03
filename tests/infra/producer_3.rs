use crate::infra::input_events::EventType3;
use crate::infra::input_events::InputEvents;
use chanix::producer::Producer;
use crossbeam::channel::{Receiver, Sender};
use log::debug;
use log::warn;
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

    fn start(&self, sender: Sender<InputEvents>, shutdown: Receiver<()>) {
        let name = self.name.clone();
        let start_id = self.start_id;
        let end_id = self.end_id;

        thread::spawn(move || {
            debug!("[Producer: {name}] Started.");

            for id in start_id..=end_id {
                crossbeam::select! {
                    recv(shutdown) -> _ => {
                        debug!("[Producer: {name}] Shutting down.");
                        break;
                    }
                    default => {
                        let message = InputEvents::EventType3(EventType3::new(id, id));

                        debug!("[Producer: {name}] Sending: {:?}", message);
                        if sender.send(message).is_err() {
                            warn!("[Producer: {name}] Failed to send a message. The channel is closed.");
                            break;
                        }
                    }
                }
            }

            debug!("[Producer: {name}] Finished.");
        });
    }
}
