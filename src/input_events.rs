#[derive(Debug, Clone)]
pub struct EventType1 {
    pub id: u32,
    pub value: String,
}

impl EventType1 {
    pub fn new(id: u32, value: &str) -> Self {
        EventType1 {
            id,
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventType2 {
    pub id: u32,
    pub amount: f64,
}

impl EventType2 {
    pub fn new(id: u32, amount: f64) -> Self {
        EventType2 { id, amount }
    }
}

#[derive(Debug, Clone)]
pub struct EventType3 {
    pub id: u32,
    pub amount: u32,
}

impl EventType3 {
    pub fn new(id: u32, amount: u32) -> Self {
        EventType3 { id, amount }
    }
}

#[derive(Debug, Clone)]
pub enum InputEvents {
    EventType1(EventType1),
    EventType2(EventType2),
    EventType3(EventType3),
}
