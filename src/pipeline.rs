use crate::aggregator::Aggregator;
use crate::consumer::Consumer;
use crate::types::{AggregationLogic, ArcedProducer};
use crossbeam::channel::{self, Receiver, Sender};
use log::{debug, error};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    producer_sender: Sender<Input>, // Sends events from producers
    producer_receiver: Option<Receiver<Input>>, // Receives events from producers
    consumer_senders: Arc<Mutex<Vec<Sender<Result>>>>, // Sends aggregated results to each consumer
    shutdown_senders: Arc<Mutex<Vec<Sender<()>>>>, // Sends shutdown signal to all components
    shutdown_receiver: Option<Receiver<()>>, // Receives shutdown signal in pipeline task
    task_handle: Option<thread::JoinHandle<()>>, // Handle for the main task loop
    aggregator: Aggregator<Input, State, Result>,
    consumers: Vec<Consumer<Result>>, // Consumers connected to the pipeline
}
impl<Input, State, Result> Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    pub fn add_producer(
        &mut self,
        producer: ArcedProducer<Input>,
        logic: AggregationLogic<Input, State, Result>,
    ) -> &mut Self {
        self.aggregator.add_producer(producer, logic);
        self
    }

    pub fn subscribe(&mut self, consumer: Consumer<Result>) -> &mut Self {
        self.consumers.push(consumer);
        self
    }

    /// # Panics
    /// Panics if the pipeline has already been started (i.e., `producer_receiver` is `None`).
    pub fn start(&mut self, initial_state: State) -> &mut Self {
        let producer_receiver = self
            .producer_receiver
            .take()
            .expect("Pipeline can only be started once.");
        let aggregated_senders = self.consumer_senders.clone();
        let shutdown_receiver = self
            .shutdown_receiver
            .take()
            .expect("Shutdown receiver already taken");
        let aggregator = self.aggregator.clone();

        self.task_handle = Some(thread::spawn(move || {
            Self::pipeline_task(
                &producer_receiver,
                &aggregated_senders,
                &shutdown_receiver,
                &aggregator,
                initial_state,
            );
        }));

        self.start_consumers();
        self.start_producers();

        self
    }

    /// Shuts down the pipeline and all associated producers and consumers.
    ///
    /// # Panics
    /// Panics if the `shutdown_senders` lock is poisoned.
    pub fn shutdown(mut self) {
        debug!("[Pipeline] Sending shutdown signal...");
        let senders = self.shutdown_senders.lock().unwrap();
        for sender in senders.iter() {
            let _ = sender.send(());
        }
        drop(senders);

        if let Some(handle) = self.task_handle.take()
            && let Err(err) = handle.join()
        {
            error!("[Pipeline] Error during shutdown: {err:?}");
        }

        debug!("[Pipeline] Shutdown complete.");
    }

    fn pipeline_task(
        producer_receiver: &Receiver<Input>,
        aggregated_senders: &Arc<Mutex<Vec<Sender<Result>>>>,
        shutdown_receiver: &Receiver<()>,
        aggregator: &Aggregator<Input, State, Result>,
        initial_state: State,
    ) {
        let mut current_state = Arc::new(initial_state);

        loop {
            crossbeam::select! {
                recv(producer_receiver) -> msg => {
                    if let Ok(input_event) = msg {
                         let (new_state, aggregated_result) = aggregator.aggregate_event(
                             input_event,
                             &current_state,
                         );

                         current_state = Arc::new(new_state);

                         if let Some(event) = aggregated_result {
                             let senders = aggregated_senders.lock().unwrap();
                             for sender in senders.iter() {
                                 if sender.send(event.clone()).is_err() {
                                     error!("Failed to send aggregated event to a consumer.");
                                 }
                             }
                         }
                     } else {
                         error!("[Pipeline] Producer channel closed.");
                         break;
                     }
                }
                recv(shutdown_receiver) -> _ => {
                    debug!("[Pipeline] Shutdown signal received.");
                    break;
                }
            }
        }
    }

    fn start_consumers(&self) {
        let mut senders = self.consumer_senders.lock().unwrap();
        let mut shutdown_senders = self.shutdown_senders.lock().unwrap();
        for consumer in &self.consumers {
            let (sender, receiver) = channel::unbounded();
            senders.push(sender);
            let (shutdown_sender, shutdown_receiver) = channel::unbounded();
            shutdown_senders.push(shutdown_sender);
            consumer.start(receiver, shutdown_receiver);
        }
    }

    fn start_producers(&self) {
        let mut shutdown_senders = self.shutdown_senders.lock().unwrap();
        for producer in &self.aggregator.get_producers() {
            let sender = self.producer_sender.clone();
            let (shutdown_sender, shutdown_receiver) = channel::unbounded();
            shutdown_senders.push(shutdown_sender);
            producer.start(sender, shutdown_receiver);
        }
    }
}

impl<Input, State, Result> Default for Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        let (producer_sender, producer_receiver) = channel::unbounded::<Input>();
        let (shutdown_sender, shutdown_receiver) = channel::unbounded::<()>();

        Self {
            producer_sender,
            producer_receiver: Some(producer_receiver),
            consumer_senders: Arc::new(Mutex::new(Vec::new())),
            shutdown_senders: Arc::new(Mutex::new(vec![shutdown_sender])),
            shutdown_receiver: Some(shutdown_receiver),
            task_handle: None,
            aggregator: Aggregator::default(),
            consumers: Vec::new(),
        }
    }
}

impl<Input, State, Result> Clone for Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        assert!(
            self.task_handle.is_none(),
            "Cannot clone a running Pipeline."
        );
        assert!(
            self.consumers.is_empty(),
            "Cannot clone a Pipeline with consumers."
        );

        let mut pipeline = Self::default();
        let producers = self.aggregator.get_producers_and_logic();
        for (producer, logic) in producers {
            pipeline.add_producer(producer, logic);
        }

        pipeline
    }
}
