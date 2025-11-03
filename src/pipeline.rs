use crate::aggregator::Aggregator;
use crate::consumer::Consumer;
use crate::types::{AggregationLogic, ArcedProducer};
use crossbeam::channel::{Receiver, Sender};
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
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
    shutdown_flag: Arc<AtomicBool>, // Broadcasts the shutdown signal
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

    pub fn start(&mut self, initial_state: State) -> &mut Self {
        let producer_receiver = self
            .producer_receiver
            .take()
            .expect("Pipeline can only be started once.");
        let aggregated_senders = self.consumer_senders.clone();
        let shutdown_flag = self.shutdown_flag.clone();
        let aggregator = self.aggregator.clone();

        self.task_handle = Some(thread::spawn(move || {
            Self::pipeline_task(
                producer_receiver,
                aggregated_senders,
                shutdown_flag,
                aggregator,
                initial_state,
            )
        }));

        self.start_consumers();
        self.start_producers();

        self
    }

    pub fn shutdown(mut self) {
        println!("[Pipeline] Sending shutdown signal...");
        self.shutdown_flag
            .store(true, std::sync::atomic::Ordering::SeqCst);

        if let Some(handle) = self.task_handle.take()
            && let Err(err) = handle.join()
        {
            eprintln!("[Pipeline] Error during shutdown: {:?}", err);
        }

        println!("[Pipeline] Shutdown complete.");
    }

    fn pipeline_task(
        producer_receiver: Receiver<Input>,
        aggregated_senders: Arc<Mutex<Vec<Sender<Result>>>>,
        shutdown_flag: Arc<AtomicBool>,
        aggregator: Aggregator<Input, State, Result>,
        initial_state: State,
    ) {
        let mut current_state = Arc::new(initial_state);

        loop {
            crossbeam::select! {
                recv(producer_receiver) -> msg => {
                    match msg {
                        Ok(input_event) => {
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
                        Err(_) => {
                            println!("[Pipeline] Producer channel closed.");
                            break;
                        }
                    }
                }
                default(std::time::Duration::from_millis(10)) => {
                    if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        println!("[Pipeline] Shutdown signal received.");
                        break;
                    }
                }
            }
        }
    }

    fn start_consumers(&self) {
        let mut senders = self.consumer_senders.lock().unwrap();
        for consumer in &self.consumers {
            let (sender, receiver) = crossbeam::channel::unbounded();
            senders.push(sender);
            consumer.start(receiver, self.shutdown_flag.clone());
        }
    }

    fn start_producers(&self) {
        for producer in &self.aggregator.get_producers() {
            let sender = self.producer_sender.clone();
            let shutdown_signal = self.shutdown_flag.clone();
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
        let (producer_sender, producer_receiver) = crossbeam::channel::unbounded::<Input>();

        Self {
            producer_sender,
            producer_receiver: Some(producer_receiver),
            consumer_senders: Arc::new(Mutex::new(Vec::new())),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
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
