use crossbeam::channel::Receiver;
use log::{debug, error};
use std::fmt::Debug;

use crate::types::ConsumerLogic;

#[derive(Clone)]
pub struct Consumer<Result>
where
    Result: Debug + Send + Sync + 'static,
{
    name: String,
    result_processor: ConsumerLogic<Result>,
}

impl<Result> Consumer<Result>
where
    Result: Debug + Send + Sync + 'static,
{
    pub fn new(name: &str, result_processor: ConsumerLogic<Result>) -> Self {
        Self {
            name: name.to_string(),
            result_processor,
        }
    }

    pub fn start(&self, receiver: Receiver<Result>, shutdown: Receiver<()>) {
        let name = self.name.clone();
        let result_processor = self.result_processor.clone();

        std::thread::spawn(move || {
            debug!("[Consumer: {name}] Starting...");

            loop {
                crossbeam::select! {
                    recv(receiver) -> msg => {
                        if let Ok(result_event) = msg {
                            debug!("[Consumer: {name}] Processing: {result_event:?}");
                            result_processor(result_event);
                        } else {
                            error!("[Consumer: {name}] Channel closed.");
                            break;
                        }
                    }
                    recv(shutdown) -> _ => {
                        debug!("[Consumer: {name}] Shutting down due to signal.");
                        break;
                    }
                }
            }

            debug!("[Consumer: {name}] Exited.");
        });
    }
}
