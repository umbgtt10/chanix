use std::fmt::Debug;
use std::sync::Arc;

use log::error;

use crate::types::{AggregationLogic, ArcedProducer};

pub struct Aggregator<Input, State, Result>
where
    Input: Debug + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    producers_and_logic: Vec<(ArcedProducer<Input>, AggregationLogic<Input, State, Result>)>,
}

impl<Input, State, Result> Aggregator<Input, State, Result>
where
    Input: Debug + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    pub fn add_producer(
        &mut self,
        producer: ArcedProducer<Input>,
        logic: AggregationLogic<Input, State, Result>,
    ) -> &mut Self {
        self.producers_and_logic.push((producer, logic));
        self
    }

    #[must_use]
    pub fn get_producers(&self) -> Vec<ArcedProducer<Input>> {
        self.producers_and_logic
            .iter()
            .map(|(prod, _)| prod.clone())
            .collect()
    }

    #[must_use]
    pub fn get_producers_and_logic(
        &self,
    ) -> Vec<(ArcedProducer<Input>, AggregationLogic<Input, State, Result>)> {
        self.producers_and_logic.clone()
    }

    #[must_use]
    pub fn aggregate_event(
        &self,
        input_event: Input,
        current_state: &Arc<State>,
    ) -> (State, Option<Result>) {
        let logic = self
            .producers_and_logic
            .iter()
            .find(|(prod, _)| prod.matches(&input_event));
        if let Some((_, logic)) = logic {
            (logic)(input_event, current_state)
        } else {
            error!("No aggregation logic found for event type: {input_event:?}");
            (current_state.as_ref().clone(), None)
        }
    }
}

impl<Input, State, Result> Default for Aggregator<Input, State, Result>
where
    Input: Debug + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            producers_and_logic: Vec::new(),
        }
    }
}

impl<Input, State, Result> Clone for Aggregator<Input, State, Result>
where
    Input: Debug + Send + Sync + 'static,
    State: Clone + Send + Sync + 'static,
    Result: Debug + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            producers_and_logic: self.producers_and_logic.clone(),
        }
    }
}
