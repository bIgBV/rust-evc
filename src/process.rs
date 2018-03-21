use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use std::thread;
use std::cmp::max;

extern crate rand;
extern crate rug;

use self::rug::{Assign, Integer};
use self::rand::Rng;

use super::event::{Event, EventType};

#[derive(Debug)]
pub struct Process {
    pub id: u64,
    pub prime: Integer,
    pub vec_clock: Vec<u64>,
    pub evc: Integer,
    pub receiver: Receiver<Event>,
    pub dispatch: Sender<Event>,
}

impl Process {
    /// Instantiates a new process. The vector is initiated with the given capacity and
    /// the local time for all other proceses set to 0.
    pub fn new(
        id: u64,
        prime: u64,
        receiver: Receiver<Event>,
        dispatch: Sender<Event>,
        capacity: i64,
    ) -> Process {
        let mut p = Process {
            id,
            prime: Integer::from(prime),
            vec_clock: Vec::with_capacity(capacity as usize),
            evc: Integer::from(1),
            receiver,
            dispatch,
        };

        // arrays are a pain, but since the size is never going to change, we can just pre-populate
        // the vector.
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
            match self.receiver.recv_timeout(Duration::from_millis(2000)) {
                Ok(mut event) => {
                    event.event_type = EventType::Receive;
                    self.handle_event(event)
                }
                Err(e) => match e {
                    RecvTimeoutError::Timeout => self.generate_event(),
                    RecvTimeoutError::Disconnected => break, // Break out of loop, effectively shutting down thread.
                },
            }
        }

        info!("Dispatcher stopped. Probably because max_bits is hit")
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
    /// calls the appropriate event handler based on the event type.
    fn handle_event(&mut self, event: Event) {
        match event.event_type {
            EventType::Internal => {
                // Update the vector clock
                self.update_vector_clock(&event);

                // update the encoded vector clock.
                let temp = self.update_encoded_clock(&event);
                self.evc.assign(temp);
                self.handle_internal()
            }
            EventType::Message => {
                self.update_vector_clock(&event);

                let temp = self.update_encoded_clock(&event);
                self.evc.assign(temp);
                self.handle_message()
            }
            EventType::Receive => {
                self.update_vector_clock(&event);

                let temp = self.update_encoded_clock(&event);
                self.evc.assign(temp);

                self.handle_receive(event)
            }
        }

        debug!(
            "p: {} Vec clock: {:?}, encoded clock: {}",
            self.id,
            self.vec_clock,
            self.evc.significant_bits()
        );
    }

    /// Updates the processe's encoded clock according to the following rules:
    /// 
    /// (1) Initialize ti = 1.
    /// (2) Before an internal event happens at process Pi ,
    ///     ti = ti ∗ pi (local tick).
    /// (3) Before process Pi sends a message, it  rst executes
    ///     ti = ti ∗ pi (local tick), then it sends the message pig-
    ///     gybacked with ti .
    /// (4) When process Pi receives a message piggybacked with
    ///     timestamp s, it executes
    ///     ti =LCM(s,ti)(merge);
    ///     ti = ti ∗ pi (local tick)
    ///     before delivering the message.
    fn update_encoded_clock(&self, event: &Event) -> Integer {
        match event.event_type {
            EventType::Receive => {
                let temp = self.evc.lcm_ref(&event.encoded_clock);
                Integer::from(temp)
            }
            _ => {
                let temp = &self.evc * &self.prime;
                Integer::from(temp)
            }
        }
    }

    /// Updates the vector clock based on the following rules:
    /// 
    /// (1) Before an internal event happens at process `Pi`, `V[i] = V[i]+1` (local tick).
    /// (2) Before process Pi sends a message, it first executes `V[i] = V [i] + 1` (local tick),
    ///     then it sends the message piggybacked with V.
    /// (3) When process Pi receives a message piggybacked with times- tamp U , it executes
    ///     ```
    ///     ∀k ∈[1...n],V[k]=max(V[k],U[k])(merge);
    ///     V[i] = V[i] + 1 (local tick)
    ///     ```
    ///     before delivering the message.
    fn update_vector_clock(&mut self, event: &Event) {
        match event.event_type {
            EventType::Receive => {
                let temp_vec = self.vec_clock
                    .iter()
                    .zip(event.vec_clock.iter())
                    .map(|(x, y)| max(*x, *y))
                    .collect();
                self.vec_clock = temp_vec;
                self.vec_clock[self.id as usize] += 1;
            }
            _ => self.vec_clock[self.id as usize] += 1,
        }
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
        thread::sleep(Duration::from_millis(2000));
    }
}
