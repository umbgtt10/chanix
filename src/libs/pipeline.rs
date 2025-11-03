use crate::libs::aggregator::Aggregator;
use crate::libs::consumer::Consumer;
use crate::libs::types::{AggregationLogic, ArcedProducer};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};
use tokio::task;

pub struct Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    producer_sender: mpsc::UnboundedSender<Input>, // Sends events from producers
    producer_receiver: Option<mpsc::UnboundedReceiver<Input>>, // Receives events from producers
    consumer_senders: Arc<Mutex<Vec<mpsc::UnboundedSender<Result>>>>, // Sends aggregated results to each consumer
    shutdown_sender: watch::Sender<bool>, // Broadcasts the shutdown signal
    task_handle: Option<tokio::task::JoinHandle<()>>, // Handle for the main task loop
    aggregator: Aggregator<Input, State, Result>,
    consumers: Vec<Consumer<Result>>, // Consumers connected to the pipeline
}

impl<Input, State, Result> Pipeline<Input, State, Result>
where
    Input: Debug + Clone + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    pub fn subscribe(&mut self, consumer: Consumer<Result>) -> &mut Self {
        self.consumers.push(consumer);
        self
    }

    pub fn add_producer(
        &mut self,
        producer: ArcedProducer<Input>,
        logic: AggregationLogic<Input, State, Result>,
    ) -> &mut Self {
        self.aggregator.add_producer(producer, logic);
        self
    }

    pub fn start(&mut self, initial_state: State) -> &mut Self {
        let producer_receiver = self
            .producer_receiver
            .take()
            .expect("Pipeline can only be started once.");
        let aggregated_senders = self.consumer_senders.clone();
        let shutdown_receiver = self.shutdown_sender.subscribe();
        let aggregator = self.aggregator.clone();

        self.task_handle = Some(task::spawn(Self::pipeline_task(
            producer_receiver,
            aggregated_senders,
            shutdown_receiver,
            aggregator,
            initial_state,
        )));

        self.start_consumers();
        self.start_producers();

        self
    }

    pub async fn shutdown(mut self) {
        println!("[Pipeline] Sending shutdown signal...");
        self.shutdown_sender.send(true).unwrap();

        if let Some(handle) = self.task_handle.take()
            && let Err(err) = handle.await
        {
            eprintln!("[Pipeline] Error during shutdown: {:?}", err);
        }

        println!("[Pipeline] Shutdown complete.");
    }

    async fn pipeline_task(
        mut producer_receiver: mpsc::UnboundedReceiver<Input>,
        aggregated_senders: Arc<Mutex<Vec<mpsc::UnboundedSender<Result>>>>,
        mut shutdown_receiver: watch::Receiver<bool>,
        aggregator: Aggregator<Input, State, Result>,
        initial_state: State,
    ) {
        let mut current_state = Arc::new(initial_state);

        loop {
            tokio::select! {
                Some(input_event) = producer_receiver.recv() => {
                    let (new_state, aggregated_result) = aggregator.aggregate_event(
                        input_event,
                        &current_state,
                    );

                    current_state = Arc::new(new_state);

                    if let Some(event) = aggregated_result {
                        let senders = aggregated_senders.lock().unwrap();
                        for sender in senders.iter() {
                            if sender.send(event.clone()).is_err() {
                                eprintln!("Failed to send aggregated event to a consumer.");
                            }
                        }
                    }
                }

                _ = shutdown_receiver.changed() => {
                    println!("[Pipeline] Shutdown signal received.");
                    break;
                }
            }
        }
    }

    fn start_consumers(&self) {
        let mut senders = self.consumer_senders.lock().unwrap();
        for consumer in &self.consumers {
            let (sender, receiver) = mpsc::unbounded_channel();
            senders.push(sender);
            consumer.start(receiver, self.shutdown_sender.subscribe());
        }
    }

    fn start_producers(&self) {
        for producer in &self.aggregator.get_producers() {
            let sender = self.producer_sender.clone();
            let shutdown_signal = self.shutdown_sender.subscribe();
            producer.start(sender, shutdown_signal);
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
        let (producer_sender, producer_receiver) = mpsc::unbounded_channel::<Input>();
        let (shutdown_sender, _) = watch::channel(false);

        Self {
            producer_sender,
            producer_receiver: Some(producer_receiver),
            consumer_senders: Arc::new(Mutex::new(Vec::new())),
            shutdown_sender,
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
        if self.task_handle.is_some() {
            panic!("Cannot clone a running Pipeline.");
        }

        if !self.consumers.is_empty() {
            panic!("Cannot clone a Pipeline with consumers.");
        }

        let mut pipeline = Self::default();
        let producers = self.aggregator.get_producers_and_logic();
        for (producer, logic) in producers {
            pipeline.add_producer(producer, logic);
        }

        pipeline
    }
}
