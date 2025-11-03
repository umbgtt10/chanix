use crate::producer::Producer;
use std::sync::Arc;

pub type AggregationLogic<Input, State, Result> =
    Arc<dyn Fn(Input, &State) -> (State, Option<Result>) + Send + Sync + 'static>;

pub type ConsumerLogic<Result> = Arc<dyn Fn(Result) + Send + Sync + 'static>;

pub type ArcedProducer<Input> = Arc<dyn Producer<Input> + Send + Sync>;
