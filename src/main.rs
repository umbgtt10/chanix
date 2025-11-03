/*
use chanix::aggregated_event::AggregatedEvent;
use chanix::input_events::InputEvents;
use chanix::libs::consumer::Consumer;
use chanix::libs::pipeline::Pipeline;
use chanix::producer_1::Producer1;
use chanix::producer_2::Producer2;
use chanix::producer_3::Producer3;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

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

pub fn consumer_logic1(event: AggregatedEvent) {
    println!("[Consumer 1] Processed: {:?}", event);
}

pub fn consumer_logic2(event: AggregatedEvent) {
    println!("[Consumer 2] Processed: {:?}", event);
}

pub fn consumer_logic3(event: AggregatedEvent) {
    println!("[Consumer 3] Processed: {:?}", event);
}

pub fn consumer_logic4(event: AggregatedEvent) {
    println!("[Consumer 4] Processed: {:?}", event);
}

#[tokio::main]
async fn main() {
    let producer_1 = Producer1::new("Producer 1", 1, 10);
    let producer_2 = Producer2::new("Producer 2", 11, 20);
    let producer_3 = Producer3::new("Producer 3", 21, 30);

    // Step 1: Create and configure the pipeline
    let mut pipeline1 = Pipeline::default();

    pipeline1
        .add_producer(
            Arc::new(producer_1),
            Arc::new(aggregation_logic_event_type1),
        )
        .add_producer(
            Arc::new(producer_2),
            Arc::new(aggregation_logic_event_type2),
        )
        .add_producer(
            Arc::new(producer_3),
            Arc::new(aggregation_logic_event_type3),
        );

    let mut pipeline2 = pipeline1.clone();

    pipeline1
        .subscribe(Consumer::new("Consumer 1", Arc::new(consumer_logic1)))
        .subscribe(Consumer::new("Consumer 2", Arc::new(consumer_logic2)))
        .start(AggregatedEvent::default());

    pipeline2
        .subscribe(Consumer::new("Consumer 3", Arc::new(consumer_logic3)))
        .subscribe(Consumer::new("Consumer 4", Arc::new(consumer_logic4)))
        .start(AggregatedEvent::default());

    // Let the pipeline run for 5 seconds
    sleep(Duration::from_secs(5)).await;

    // Step 2: Shutdown the pipeline
    pipeline1.shutdown().await;
    pipeline2.shutdown().await;
}
 */

fn main() {
    println!("Chanix library - no main executable defined.");
}
