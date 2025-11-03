use crate::infra::{
    aggregated_event::AggregatedEvent, input_events::InputEvents, producer_1::Producer1,
    producer_2::Producer2, producer_3::Producer3,
};
use chanix::{consumer::Consumer, pipeline::Pipeline};
use log::info;
use std::{
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

mod infra;

pub fn aggregation_logic_event_type1(
    event: InputEvents,
    previous_state: &AggregatedEvent,
) -> (AggregatedEvent, Option<AggregatedEvent>) {
    if let InputEvents::EventType1(event) = event {
        let result = AggregatedEvent::new_string(&event.value.clone(), previous_state);
        let new_state = result.clone();
        (new_state, Some(result))
    } else {
        panic!("Invalid event type provided to logic_event_type1");
    }
}

pub fn aggregation_logic_event_type2(
    event: InputEvents,
    previous_state: &AggregatedEvent,
) -> (AggregatedEvent, Option<AggregatedEvent>) {
    if let InputEvents::EventType2(event) = event {
        let result = AggregatedEvent::new_float(event.amount, previous_state);
        let new_state = result.clone();
        (new_state, Some(result))
    } else {
        panic!("Invalid event type provided to logic_event_type2");
    }
}

pub fn aggregation_logic_event_type3(
    event: InputEvents,
    previous_state: &AggregatedEvent,
) -> (AggregatedEvent, Option<AggregatedEvent>) {
    if let InputEvents::EventType3(event) = event {
        let result = AggregatedEvent::new_int(event.amount, previous_state);
        let new_state = result.clone();
        (new_state, Some(result))
    } else {
        panic!("Invalid event type provided to logic_event_type3");
    }
}

static AGGREGATED_RESULT_EVENT: LazyLock<AggregatedEvent> = LazyLock::new(|| AggregatedEvent {
    int: 30,
    float: 20.0,
    string: "Value 5".to_string(),
    count: 25,
});

static CONSUMER_1_FLAG: AtomicBool = AtomicBool::new(false);
static CONSUMER_2_FLAG: AtomicBool = AtomicBool::new(false);

pub fn consumer_logic1(event: AggregatedEvent) {
    info!("[Consumer 1] Processed: {:?}", event);

    if event == *AGGREGATED_RESULT_EVENT {
        CONSUMER_1_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn consumer_logic2(event: AggregatedEvent) {
    info!("[Consumer 2] Processed: {:?}", event);

    if event == *AGGREGATED_RESULT_EVENT {
        CONSUMER_2_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[test]
fn pipeline_three_producers_two_consumers_test() {
    let _ = env_logger::builder().is_test(true).try_init();

    // Arrange
    let producer1 = Producer1::new("Producer1", 1, 5);
    let producer2 = Producer2::new("Producer2", 11, 20);
    let producer3 = Producer3::new("Producer3", 21, 30);
    let consumer1 = Consumer::new("Consumer1", Arc::new(consumer_logic1));
    let consumer2 = Consumer::new("Consumer2", Arc::new(consumer_logic2));
    let mut pipeline = Pipeline::default();

    pipeline
        .add_producer(Arc::new(producer1), Arc::new(aggregation_logic_event_type1))
        .add_producer(Arc::new(producer2), Arc::new(aggregation_logic_event_type2))
        .add_producer(Arc::new(producer3), Arc::new(aggregation_logic_event_type3))
        .subscribe(consumer1)
        .subscribe(consumer2)
        .start(AggregatedEvent::default());

    // Act
    thread::sleep(Duration::from_millis(50));

    // Assert
    assert!(CONSUMER_1_FLAG.load(Ordering::SeqCst));
    assert!(CONSUMER_2_FLAG.load(Ordering::SeqCst));

    // Clean up
    pipeline.shutdown();
}
