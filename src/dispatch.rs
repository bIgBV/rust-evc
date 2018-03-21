use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

extern crate rand;

use self::rand::Rng;

use super::event::{Event, EventType};
use super::config::Config;

type ProcessMap = HashMap<u64, Sender<Event>>;

pub struct Dispatch<'a> {
    receiver: Receiver<Event>,
    process_map: ProcessMap,
    process_ids: Vec<u64>,
    counter: u64,
    config: &'a Config,
}

impl<'a> Dispatch<'a> {
    pub fn new(receiver: Receiver<Event>, process_map: ProcessMap, config: &'a Config) -> Dispatch<'a> {
        let process_ids = process_map.keys().map(|k| k.clone()).collect();

        Dispatch {
            receiver,
            process_map,
            process_ids,
            counter: 0,
            config
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
                break
            }
            self.counter += 1;

            if event.event_type == EventType::Message {
                if let Some(process_id) = rand::thread_rng().choose(&self.process_ids) {
                    if let Some(sender) = self.process_map.get(process_id) {
                        info!("Send from: {} Dispatching to : {}", event.process_id, process_id);
                        sender
                            .send(event)
                            .unwrap_or_else(|e| error!("Error dispatcing: {}", e));
                    }
                } else {
                    panic!("Unable to select thread to dispatch to");
                }
            }
        }
    }
}
