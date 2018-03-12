use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use std::thread;

use super::event::{Event, EventType};

#[derive(Debug)]
pub struct Process {
    pub id: u64,
    pub prime: u64,
    pub clock: Vec<u64>,
    pub receiver: Receiver<Event>,
}

impl Process {
    pub fn new(id: u64, prime: u64, receiver: Receiver<Event>) -> Process {
        Process {
            id,
            prime,
            clock: Vec::new(),
            receiver,
        }
    }

    /// The main driver of a process. It waits for a message to arrive on the
    /// channel for 500 milliseconds. If there is no message, then it will either
    /// perform an intenral event or a send send event, this will be randomly
    /// selected.
    pub fn handle_dispatch(&mut self) {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    self.handle_event(&event)
                }
                Err(e) => match e {
                    RecvTimeoutError::Timeout => thread::sleep(Duration::from_millis(500)),
                    RecvTimeoutError::Disconnected => break,
                },
            }
        }
    }

    /// Waits for events to be added to the queue and handles them appropriately
    fn handle_event(&self, event: &Event) {
        match event.event_type {
            EventType::Internal => info!("Internal event!"),
            EventType::Message => info!("Send event!"),
            EventType::Receive => info!("Receive event")
        }
    }
}
