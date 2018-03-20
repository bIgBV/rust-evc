
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
    pub process_id: u64,
    pub vec_clock: Vec<u64>,
    pub encoded_clock: Integer,
    pub event_type: EventType,
}

impl Event {
    pub fn new(event_type: EventType, clock: &Vec<u64>, evc: &Integer, process_id: u64) -> Event {
        Event {
            process_id,
            vec_clock: clock.clone(),
            encoded_clock: evc.clone(),
            event_type,
        }
    }
}
