use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use std::thread;
use std::cmp::max;
use std::sync::Arc;

extern crate rand;
extern crate rug;

use self::rug::{Assign, Float, Integer};
use self::rand::Rng;

use config::Config;
use event::{next_event_id, Event, EventType};

#[derive(Debug)]
pub struct Process {
    pub id: u64,
    pub prime: Integer,
    pub vec_clock: Vec<u64>,
    pub evc: Integer,
    pub log_evc: Float,
    pub receiver: Receiver<Event>,
    pub dispatch: Sender<Event>,
    pub collector: Sender<Event>,
    config: Arc<Config>,
}

impl Process {
    /// Instantiates a new process. The vector is initiated with the given capacity and
    /// the local time for all other process set to 0.
    pub fn new(
        id: u64,
        prime: u64,
        receiver: Receiver<Event>,
        dispatch: Sender<Event>,
        collector: Sender<Event>,
        config: Arc<Config>,
    ) -> Process {
        let mut p = Process {
            id,
            prime: Integer::from(prime),
            vec_clock: Vec::with_capacity(config.num_processes as usize),
            evc: Integer::from(1),
            log_evc: Float::with_val(config.float_precision, 0),
            receiver,
            dispatch,
            collector,
            config,
        };

        // arrays are a pain, but since the size is never going to change, we can just pre-populate
        // the vector.
        for _ in 0..p.config.num_processes {
            p.vec_clock.push(0);
        }

        p.vec_clock[p.id as usize] = 1;

        p
    }

    /// The main driver of a process. It waits for a message to arrive on the
    /// channel for 500 milliseconds. If there is no message, then it will either
    /// perform an internal event or a send send event, this will be randomly
    /// selected
    pub fn handle_dispatch(&mut self) {
        loop {
            match self.receiver
                .recv_timeout(Duration::from_millis(self.config.timeout))
            {
                Ok(event) => {
                    let receive_event = Event::new(
                        EventType::Receive,
                        next_event_id(),
                        &event.vec_clock,
                        &event.encoded_clock,
                        &event.log_encoded_clock,
                        event.process_id,
                    );
                    self.handle_event(receive_event)
                }
                Err(e) => match e {
                    RecvTimeoutError::Timeout => self.generate_event(),
                    RecvTimeoutError::Disconnected => break, // Break out of loop, effectively shutting down thread.
                },
            }
        }

        info!("p: {} closing process", self.id)
    }

    /// Event creator.
    fn create_event(&self, event_type: EventType) -> Event {
        Event::new(
            event_type,
            next_event_id(),
            &self.vec_clock,
            &self.evc,
            &self.log_evc,
            self.id,
        )
    }

    /// Randomly generates events when a message is not received across a channel.
    fn generate_event(&mut self) {
        let event_choices = vec![EventType::Send, EventType::Internal];
        if let Some(event_choice) = rand::thread_rng().choose(&event_choices) {
            match *event_choice {
                EventType::Internal => {
                    let event = self.create_event(EventType::Internal);
                    self.handle_event(event)
                }
                EventType::Send => {
                    let event = self.create_event(EventType::Send);
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
        // Update the vector clock
        self.update_vector_clock(&event);

        // update the encoded vector clock.
        let temp = self.update_encoded_clock(&event);
        self.evc.assign(temp);

        self.log_evc = self.update_log_encoded_clock(&event);

        match event.event_type {
            EventType::Internal => self.handle_internal(),
            EventType::Send => self.handle_message(),
            EventType::Receive => self.handle_receive(event),
        }

        //        debug!(
        //            "p: {} Vec clock: {:?}, encoded clock: {}, log encoded clock: {}",
        //            self.id,
        //            self.vec_clock,
        //            self.evc.significant_bits(),
        //            self.log_evc
        //        );
    }

    /// Updates the process's encoded clock according to the following rules:
    ///
    ///* Initialize ti = 1.
    ///* Before an internal event happens at process Pi ,
    ///  ti = ti ∗ pi (local tick).
    ///* Before process Pi sends a message, it  rst executes
    ///  ti = ti ∗ pi (local tick), then it sends the message piggybacked with ti .
    ///* When process Pi receives a message piggybacked with
    ///  timestamp s, it executes
    ///  ti =LCM(s,ti)(merge);
    ///  ti = ti ∗ pi (local tick)
    ///  before delivering the message.
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

    fn update_log_encoded_clock(&self, event: &Event) -> Float {
        match event.event_type {
            EventType::Receive => {
                // merge two encoded clocks
                let log_evc_anti =
                    Float::with_val(self.config.float_precision, self.log_evc.exp2_ref());
                let ts_anti = Float::with_val(
                    self.config.float_precision,
                    event.log_encoded_clock.exp2_ref(),
                );

                let evc_anti_int = match log_evc_anti.to_integer() {
                    Some(i) => i,
                    None => unreachable!(),
                };

                let ts_anti_int = match ts_anti.to_integer() {
                    Some(i) => i,
                    None => unreachable!(),
                };

                let ts_gcd = evc_anti_int.gcd(&ts_anti_int);

                let ts_gcd_fl = Float::with_val(self.config.float_precision, ts_gcd);

                let temp = Float::with_val(
                    self.config.float_precision,
                    &event.log_encoded_clock + &self.log_evc,
                );
                Float::with_val(self.config.float_precision, temp - (ts_gcd_fl.log2()))
            }
            _ => {
                // Tick of the clock
                let prime_fl = Float::with_val(self.config.float_precision, &self.prime);
                let log_prime_fl = prime_fl.log2();
                Float::with_val(self.config.float_precision, &self.log_evc + &log_prime_fl)
            }
        }
    }

    /// Updates the vector clock based on the following rules:
    ///
    /// * Before an internal event happens at process `Pi`, `V[i] = V[i]+1` (local tick).
    /// * Before process Pi sends a message, it first executes `V[i] = V [i] + 1` (local tick),
    ///   then it sends the message piggybacked with V.
    /// * When process Pi receives a message piggybacked with times- tamp U , it executes
    ///
    ///   ```text
    ///   ∀k ∈[1...n],V[k]=max(V[k],U[k])(merge);
    ///   V[i] = V[i] + 1 (local tick)
    ///   ```
    ///
    ///   before delivering the message.
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

    /// Logs a receive event.
    fn handle_receive(&self, event: Event) {
        debug!(
            "p: {}: Received event: {:?} from {}",
            self.id, event, event.process_id
        );
        self.collector
            .send(Message(Event))
            .unwrap_or_else(|e| error!("p: {} error sending event to collector: {}", self.id, e));
    }

    /// Sends a message to the dispatcher to be sent to a random process.
    fn handle_message(&self) {
        let message = Event::new(
            EventType::Send,
            next_event_id(),
            &self.vec_clock,
            &self.evc,
            &self.log_evc,
            self.id,
        );
        debug!("p: {}: Dispatching event: {:?}", self.id, message);
        self.dispatch
            .send(message)
            .unwrap_or_else(|e| error!("p: {}: error sending message: {}", self.id, e));
        self.collector
            .send(event)
            .unwrap_or_else(|e| error!("p: {} error sending event to collector: {}", self.id, e));
    }

    /// Simulating an internal process. The process sleeps for a random amount of time
    fn handle_internal(&self) {
        let message = Event::new(
            EventType::Internal,
            next_event_id(),
            &self.vec_clock,
            &self.evc,
            &self.log_evc,
            self.id,
        );
        info!("p: {}: internal event", self.id);
        debug!("p: {}: Performing internal event: {:?}", self.id, message);
        self.collector
            .send(event)
            .unwrap_or_else(|e| error!("p: {} error sending event to collector: {}", self.id, e));
        thread::sleep(Duration::from_millis(self.config.timeout));
    }
}
