extern crate rug;

use self::rug::{Float, Integer};

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
    pub log_encoded_clock: Float,
    pub event_type: EventType,
}

impl Event {
    pub fn new(
        event_type: EventType,
        clock: &[u64],
        evc: &Integer,
        log_evc: &Float,
        process_id: u64,
    ) -> Event {
        Event {
            process_id,
            vec_clock: clock.to_owned(),
            encoded_clock: evc.clone(),
            log_encoded_clock: log_evc.clone(),
            event_type,
        }
    }
}
