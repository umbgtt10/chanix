use std::fmt::Debug;
use tokio::sync::{mpsc, watch};

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
    /// Create a new consumer
    pub fn new(name: &str, result_processor: ConsumerLogic<Result>) -> Self {
        Self {
            name: name.to_string(),
            result_processor,
        }
    }

    pub fn start(
        &self,
        mut receiver: mpsc::UnboundedReceiver<Result>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let name = self.name.clone();
        let result_processor = self.result_processor.clone();

        tokio::spawn(async move {
            println!("[Consumer: {name}] Starting...");

            loop {
                tokio::select! {
                    // Process incoming events
                    Some(result_event) = receiver.recv() => {
                        println!("[Consumer: {name}] Processing: {:?}", result_event);
                        result_processor(result_event); // Call the event processor
                    }

                    // Handle shutdown signal
                    _ = shutdown.changed() => {
                        println!("[Consumer: {name}] Shutting down due to signal.");
                        break;
                    }
                }
            }

            println!("[Consumer: {name}] Exited.");
        });
    }
}
