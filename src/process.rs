use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use std::thread;
use std::cmp::max;

extern crate rand;
extern crate rug;

use self::rug::Integer;
use self::rand::Rng;

use super::event::{Event, EventType};

#[derive(Debug)]
pub struct Process {
    pub id: u64,
    pub prime: u64,
    pub vec_clock: Vec<u64>,
    pub evc: Integer,
    pub receiver: Receiver<Event>,
    pub dispatch: Sender<Event>,
}

impl Process {
    pub fn new(
        id: u64,
        prime: u64,
        receiver: Receiver<Event>,
        dispatch: Sender<Event>,
        capacity: i64,
    ) -> Process {
        let mut p = Process {
            id,
            prime,
            vec_clock: Vec::with_capacity(capacity as usize),
            evc: Integer::new(),
            receiver,
            dispatch,
        };

        for _ in 0..capacity {
            p.vec_clock.push(0);
        }

        p.vec_clock[p.id as usize] = 1;

        p
    }

    /// The main driver of a process. It waits for a message to arrive on the
    /// channel for 500 milliseconds. If there is no message, then it will either
    /// perform an intenral event or a send send event, this will be randomly
    /// selected.
    pub fn handle_dispatch(&mut self) {
        loop {
            match self.receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(event) => self.handle_event(event),
                Err(e) => match e {
                    RecvTimeoutError::Timeout => self.generate_event(),
                    RecvTimeoutError::Disconnected => break, // Break out of loop, effectively shutting down thread.
                },
            }
        }
    }

    /// Event creator.
    fn create_event(&self, event_type: EventType) -> Event {
        Event::new(event_type, &self.vec_clock, &self.evc, self.id)
    }

    /// Randomly generates events when a message is not received across a channel.
    fn generate_event(&mut self) {
        let event_choices = vec![EventType::Message, EventType::Internal];
        if let Some(event_choice) = rand::thread_rng().choose(&event_choices) {
            match event_choice {
                &EventType::Internal => {
                    let event = self.create_event(EventType::Internal);
                    self.handle_event(event)
                }
                &EventType::Message => {
                    let event = self.create_event(EventType::Message);
                    self.handle_event(event)
                }
                _ => info!("Just to make the type checker happy"),
            }
        } else {
            error!("Unable to choose event type");
        }
    }

    /// Main even handler. This updates both the vector clock and the encoded clock and
    /// calls the appropriate event handler based on the type.
    fn handle_event(&mut self, event: Event) {
        match event.event_type {
            EventType::Internal => {
                self.vec_clock[self.id as usize] += 1;
                self.handle_internal()
            }
            EventType::Message => {
                self.vec_clock[self.id as usize] += 1;
                self.handle_message()
            }
            EventType::Receive => {
                let mut temp_vec = Vec::new();
                {
                    let it = event.vec_clock.iter().zip(self.vec_clock.iter());

                    for (i, (x, y)) in it.enumerate() {
                        temp_vec[i] = max(*x, *y);
                    }
                }
                self.vec_clock = temp_vec;
                self.vec_clock[self.id as usize] += 1;
                self.handle_receive(event)
            }
        }

        info!("p: {} Vec clock: {:?}", self.id, self.vec_clock);
    }

    fn handle_receive(&self, event: Event) {
        info!("p: {}: Received event from {}", self.id, event.process_id);
    }

    fn handle_message(&self) {
        info!("p: {}: Dispatching message", self.id);
        self.dispatch
            .send(Event::new(
                EventType::Message,
                &self.vec_clock,
                &self.evc,
                self.id,
            ))
            .unwrap_or_else(|e| error!("p: {}: error sending message: {}", self.id, e));
    }

    fn handle_internal(&self) {
        info!("p: {}: Performing intenral event", self.id);
        thread::sleep(Duration::from_millis(500));
    }
}
