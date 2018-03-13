use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

extern crate rand;

use self::rand::Rng;

use super::event::Event;

type ProcessMap = HashMap<u64, Sender<Event>>;

pub struct Dispatch {
    receiver: Receiver<Event>,
    process_map: ProcessMap,
    process_ids: Vec<u64>,
}

impl Dispatch {
    pub fn new(receiver: Receiver<Event>, process_map: ProcessMap) -> Dispatch {
        let process_ids = process_map.keys().map(|k| k.clone()).collect();

        Dispatch {
            receiver,
            process_map,
            process_ids,
        }
    }

    pub fn handle_dispatch(&mut self) {
        for event in self.receiver.iter() {
            if let Some(process_id) = rand::thread_rng().choose(&self.process_ids) {
                if let Some(sender) = self.process_map.get(process_id) {
                    info!("Dispatching to process: {}", process_id);
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
