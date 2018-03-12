use std::thread::spawn;
use std::fs::File;

#[macro_use]
extern crate log;
extern crate rug;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;

use rug::Float;
use rug::Integer;
use rug::ops::Pow;

mod utils;
mod config;

use config::parse_config;
use utils::nth_prime;

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

fn run_thread(process: Process) {
    info!("Thread: {}, prime: {}", process.id, process.prime);
}

fn main() {
    let config = match parse_config("/Users/bIgB/.config/rust-evc/config.toml") {
        Ok(c) => c,
        Err(e) => {
            error!("Unable to read config");
            panic!("Error: {}", e);
        }
    };

    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(simplelog::LevelFilter::Info, simplelog::Config::default())
            .expect("Failed to create term logger"),
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            File::create("rust-evc.log").expect("Failed to create log file"),
        ),
    ]).expect("Failed to init logger");

    info!("Setting up with {} threads", config.num_processes);

    let mut thread_handles = vec![];
    for i in 1..config.num_processes + 1 {
        let process = Process::new(i as u64, nth_prime(i as u64));

        thread_handles.push(spawn(move || run_thread(process)));
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}
