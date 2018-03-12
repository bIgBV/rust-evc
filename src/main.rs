#[macro_use]
extern crate serde_derive;
extern crate rug;

use rug::Float;
use rug::Integer;
use rug::ops::Pow;

mod utils;
mod config;

#[derive(Debug)]
struct Process {
    pub id: u64,
    pub prime: u64,
    pub clock: Vec<u64>,
}

impl Process {
    pub fn new(id: u64, prime: u64) -> Process {
        Process {
            id: id,
            prime: prime,
            clock: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum EventType {
    Internal,
    Message,
    Receive,
}

#[derive(Debug)]
struct Event {
    pub vec_clock: Vec<u64>,
    pub encoded_clock: Integer,
    pub event_type: EventType,
}

impl Event {
    pub fn new(clock: Vec<u64>, evc: Integer, event_type: EventType) -> Event {
        Event {
            vec_clock: clock.clone(),
            encoded_clock: evc.clone(),
            event_type,
        }
    }
}

fn main() {
    let valid = Float::parse("3030695816197385100561300861328125000000147287927682734987138769");

    let f = Float::with_val(64, valid.unwrap());

    let log10 = Float::with_val(64, f.log10_ref());
    println!("parsed float: {}, it's log10: {},", f, log10);

    let anti_log = log10.exp10();
    println!("it's anti-log: {}", anti_log);

    println!("squared: {}", f.pow(20));
}
