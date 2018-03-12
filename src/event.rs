
extern crate rug;

use self::rug::Integer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Internal,
    Message,
    Receive,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub vec_clock: Vec<u64>,
    pub encoded_clock: Integer,
    pub event_type: EventType,
}

impl Event {
    pub fn new(clock: Vec<u64>, evc: Integer, event_type: EventType) -> Event {
        Event {
            vec_clock: clock.clone(),
            encoded_clock: evc.clone(),
            event_type,
        }
    }
}
