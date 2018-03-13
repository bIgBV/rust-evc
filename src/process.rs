use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use std::thread;

extern crate rand;

use self::rand::Rng;

use super::event::{Event, EventType};

#[derive(Debug)]
pub struct Process {
    pub id: u64,
    pub prime: u64,
    pub clock: Vec<u64>,
    pub receiver: Receiver<Event>,
    pub dispatch: Sender<Event>,
}

impl Process {
    pub fn new(id: u64, prime: u64, receiver: Receiver<Event>, dispatch: Sender<Event>) -> Process {
        Process {
            id,
            prime,
            clock: Vec::new(),
            receiver,
            dispatch,
        }
    }

    /// The main driver of a process. It waits for a message to arrive on the
    /// channel for 500 milliseconds. If there is no message, then it will either
    /// perform an intenral event or a send send event, this will be randomly
    /// selected.
    pub fn handle_dispatch(&mut self) {
        let event_choices = vec![EventType::Message, EventType::Internal];

        loop {
            match self.receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(event) => self.handle_receive(&event),
                Err(e) => match e {
                    RecvTimeoutError::Timeout => {
                        if let Some(event_choice) = rand::thread_rng().choose(&event_choices) {
                            match event_choice {
                                &EventType::Internal => self.handle_internal(),
                                &EventType::Message => self.handle_message(),
                                _ => info!("Just to make the type checker happy"),
                            }
                        } else {
                            error!("Unable to choose event type");
                        }
                    }
                    RecvTimeoutError::Disconnected => break,  // Break out of loop, effectively shutting down thread.
                },
            }
        }
    }

    fn handle_receive(&self, event: &Event) {
        info!("p: {}: Received event from {}", self.id, event.process_id);
    }

    fn handle_message(&self) {
        info!("p: {}: Dispatching message", self.id);
        self.dispatch
            .send(Event::new(EventType::Message, self.id))
            .unwrap_or_else(|e| error!("p: {}: error sending message: {}", self.id, e));
    }

    fn handle_internal(&self) {
        info!("p: {}: Performing intenral event", self.id);
        thread::sleep(Duration::from_millis(500));
    }
}
