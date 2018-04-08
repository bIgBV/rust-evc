use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

extern crate rand;

use self::rand::Rng;
use event::{Event, EventType};
use config::Config;
use collector::Message;

type ProcessMap = HashMap<u64, Sender<Event>>;

pub struct Dispatch {
    receiver: Receiver<Event>,
    collector: Sender<Message>,
    process_map: ProcessMap,
    process_ids: Vec<u64>,
    config: Arc<Config>,
}

impl Dispatch {
    pub fn new(
        receiver: Receiver<Event>,
        collector: Sender<Message>,
        process_map: ProcessMap,
        config: Arc<Config>,
    ) -> Dispatch {
        let process_ids = process_map.keys().cloned().collect();

        Dispatch {
            receiver,
            collector,
            process_map,
            process_ids,
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
                debug!("Closing simulation.");
                self.collector
                    .send(Message::End)
                    .unwrap_or_else(|e| error!("Error sending message to collector: {}", e));
                break;
            }

            if event.event_type == EventType::Send {
                let mut sending_process = 0;
                if let Some(position) = self.process_ids.iter().position(|x| *x == event.process_id)
                {
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
                            .unwrap_or_else(|e| error!("Error dispatching: {}", e));
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
