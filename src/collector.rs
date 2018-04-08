use event::Event;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct Collector {
    pub events: Vec<Event>,
    receiver: Receiver<Message>,
}

impl Collector {
    pub fn new(receiver: Receiver<Message>) -> Collector {
        Collector {
            events: Vec::new(),
            receiver,
        }
    }

    pub fn handle_dispatch(mut self) {
        for message in self.receiver {
            match message {
                Message::Data(e) => self.events.push(e),
                Message::End => debug!("Got {} events.", self.events.len()),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Data(Event),
    End,
}
