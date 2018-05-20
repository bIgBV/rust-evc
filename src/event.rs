extern crate rug;

use self::rug::{Float, Integer};

use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::cmp;

use collector::{Pair, ToPair};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Internal,
    Send,
    Receive,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: usize,
    pub process_id: u64,
    pub vec_clock: Vec<u64>,
    pub encoded_clock: Integer,
    pub log_encoded_clock: Float,
    pub event_type: EventType,
}

impl Event {
    pub fn new(
        event_type: EventType,
        event_id: usize,
        clock: &[u64],
        evc: &Integer,
        log_evc: &Float,
        process_id: u64,
    ) -> Event {
        Event {
            process_id,
            id: event_id,
            vec_clock: clock.to_owned(),
            encoded_clock: evc.clone(),
            log_encoded_clock: log_evc.clone(),
            event_type,
        }
    }
}

impl ToPair for Event {
    fn make_pair(self, receiver: Event) -> Pair {
        Pair {
            send: self,
            receive: receiver,
        }
    }
}

static CURRENT_EVENT_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn next_event_id() -> usize {
    CURRENT_EVENT_ID.fetch_add(1, Ordering::Relaxed)
}
