use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

extern crate rand;

use self::rand::Rng;

use super::event::{Event, EventType};
use super::config::Config;

type ProcessMap = HashMap<u64, Sender<Event>>;

pub struct Dispatch {
    receiver: Receiver<Event>,
    process_map: ProcessMap,
    process_ids: Vec<u64>,
    counter: u64,
    config: Arc<Config>,
}

impl Dispatch {
    pub fn new(
        receiver: Receiver<Event>,
        process_map: ProcessMap,
        config: Arc<Config>,
    ) -> Dispatch {
        let process_ids = process_map.keys().map(|k| k.clone()).collect();

        Dispatch {
            receiver,
            process_map,
            process_ids,
            counter: 0,
            config,
        }
    }

    /// Increments internal event counter and if it is a send event, selects a
    /// random process to send to and dispatches the event to it.
    ///
    /// If the significant bits of any encoded vector clock is greater than the
    /// config.max_bits, then it breaks out of the loop, which triggers drops all
    /// the senders of all the processes, signalling shutdown.
    pub fn handle_dispatch(&mut self) {
        for event in self.receiver.iter() {
            if event.encoded_clock.significant_bits() >= self.config.max_bits {
                debug!("Closing simulation. Events: {}", self.counter);
                break;
            }
            self.counter += 1;

            if event.event_type == EventType::Message {
                let mut sending_process = 0;
                if let Some(position) = self.process_ids.iter().position(|x| *x == event.process_id) {
                    sending_process = self.process_ids.remove(position);
                } else {
                    panic!("Unable to find process");
                }

                if let Some(process_id) = rand::thread_rng().choose(&self.process_ids) {
                    if let Some(sender) = self.process_map.get(process_id) {
                        info!(
                            "Send from: {} Dispatching to : {}",
                            event.process_id, process_id
                        );
                        sender
                            .send(event)
                            .unwrap_or_else(|e| error!("Error dispatcing: {}", e));
                        
                    }
                } else {
                    error!("Unable to select thread to dispatch to");
                    break;
                }

                self.process_ids.push(sending_process)
            }
        }
    }
}
