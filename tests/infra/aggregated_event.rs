use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct AggregatedEvent {
    pub int: u32,
    pub float: f64,
    pub string: String,
    pub count: u32,
}

impl AggregatedEvent {
    pub fn new_int(int: u32, previous: &AggregatedEvent) -> Self {
        AggregatedEvent {
            int,
            float: previous.float,
            string: previous.string.clone(),
            count: previous.count + 1,
        }
    }

    pub fn new_float(float: f64, previous: &AggregatedEvent) -> Self {
        AggregatedEvent {
            int: previous.int,
            float,
            string: previous.string.clone(),
            count: previous.count + 1,
        }
    }

    pub fn new_string(string: &str, previous: &AggregatedEvent) -> Self {
        AggregatedEvent {
            int: previous.int,
            float: previous.float,
            string: string.to_string(),
            count: previous.count + 1,
        }
    }
}

impl Default for AggregatedEvent {
    fn default() -> Self {
        AggregatedEvent {
            int: 0,
            float: 0.0,
            string: String::new(),
            count: 0,
        }
    }
}

impl Debug for AggregatedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AggregatedEvent {{ int: {}, float: {}, string: '{}', count: {} }}",
            self.int, self.float, self.string, self.count
        )
    }
}
