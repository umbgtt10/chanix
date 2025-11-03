use crate::infra::{
    aggregated_event::AggregatedEvent, input_events::InputEvents, producer_1::Producer1,
    producer_2::Producer2, producer_3::Producer3,
};
use chanix::{consumer::Consumer, pipeline::Pipeline};
use log::info;
use std::sync::Arc;

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

fn create_consumer_logic(
    expected_event: AggregatedEvent,
    completion_sender: crossbeam::channel::Sender<()>,
) -> impl Fn(AggregatedEvent) {
    move |event: AggregatedEvent| {
        info!("[Consumer] Processed: {:?}", event);

        if event == expected_event {
            let _ = completion_sender.send(());
        }
    }
}

#[test]
fn pipeline_three_producers_two_consumers_test() {
    let _ = env_logger::builder().is_test(true).try_init();

    // Arrange
    let expected_event = AggregatedEvent {
        int: 30,
        float: 20.0,
        string: "Value 5".to_string(),
        count: 25,
    };

    let (completion_sender1, completion_receiver1) = crossbeam::channel::unbounded();
    let (completion_sender2, completion_receiver2) = crossbeam::channel::unbounded();

    let consumer_logic1 = create_consumer_logic(expected_event.clone(), completion_sender1);
    let consumer_logic2 = create_consumer_logic(expected_event.clone(), completion_sender2);

    let producer1 = Producer1::new("Producer1", 1, 5);
    let producer2 = Producer2::new("Producer2", 11, 20);
    let producer3 = Producer3::new("Producer3", 21, 30);
    let consumer1 = Consumer::new("Consumer1", Arc::new(consumer_logic1));
    let consumer2 = Consumer::new("Consumer2", Arc::new(consumer_logic2));
    let mut pipeline = Pipeline::default();

    // Act
    pipeline
        .add_producer(Arc::new(producer1), Arc::new(aggregation_logic_event_type1))
        .add_producer(Arc::new(producer2), Arc::new(aggregation_logic_event_type2))
        .add_producer(Arc::new(producer3), Arc::new(aggregation_logic_event_type3))
        .subscribe(consumer1)
        .subscribe(consumer2)
        .start(AggregatedEvent::default());

    // Assert
    completion_receiver1
        .recv()
        .expect("Consumer 1 should receive the expected event");
    completion_receiver2
        .recv()
        .expect("Consumer 2 should receive the expected event");

    // Clean up
    pipeline.shutdown();
}
