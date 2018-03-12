use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

extern crate rand;

use rand::Rng;

use super::event::Event;

type ProcessMap = HashMap<u64, Sender<Event>>;

pub struct Dispatch {
    receiver: Receiver<Event>,
    process_map: ProcessMap,
    process_ids: Vec<u64>
}

impl Dispatch {
    pub fn new(receiver: Receiver<Event>, process_map: ProcessMap) -> Dispatch {
        Dispatch {
            receiver,
            process_map,
            process_ids: process_map.keys().collect(),
        }
    }

    pub fn handle_dispatch(&self) {
        for event in self.receiver {
            let process_id = rand::thread_rng().chose(&self.process_ids);
            if let Some(sender) =  self.process_map.get(process_id) {
                sender.send(event);
            }
        }
    }
}
