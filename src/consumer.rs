use crossbeam::channel::Receiver;
use std::fmt::Debug;
use std::sync::{Arc, atomic::AtomicBool};

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

    pub fn start(&self, receiver: Receiver<Result>, shutdown: Arc<AtomicBool>) {
        let name = self.name.clone();
        let result_processor = self.result_processor.clone();

        std::thread::spawn(move || {
            println!("[Consumer: {name}] Starting...");

            loop {
                crossbeam::select! {
                    recv(receiver) -> msg => {
                        match msg {
                            Ok(result_event) => {
                                println!("[Consumer: {name}] Processing: {:?}", result_event);
                                result_processor(result_event);
                            }
                            Err(_) => {
                                println!("[Consumer: {name}] Channel closed.");
                                break;
                            }
                        }
                    }
                    default(std::time::Duration::from_millis(10)) => {
                        if shutdown.load(std::sync::atomic::Ordering::SeqCst) {
                            println!("[Consumer: {name}] Shutting down due to signal.");
                            break;
                        }
                    }
                }
            }

            println!("[Consumer: {name}] Exited.");
        });
    }
}
