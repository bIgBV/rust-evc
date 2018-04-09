//! This is an implementation for the Encoded Vector Clock: Using Primes to
//! Characterize Causality in Distributed Systems by Ajay D. Kshemkalyani,
//! Ashfaq A. Khokhar, Min Shen. The application is multi-threaded with
//! various worker threads and a dispatcher thread counting all the events
//! occurring in the system as well as dispatching send events randomly between
//! the various threads.
//!
//! Each thread simulates a single process in a distributed system. It waits for a
//! receive event on it's receiver end of the channel (here the channel is used as a
//! FIFO thread safe queue) for a configurabele amount of time after which it
//! will either performa an internal event or send a message to be dispatched.
//!
//! The dispatcher in turn selects a random process and dispatches the message
//! across.

use std::thread::spawn;
use std::fs::File;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::sync::Arc;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;

mod utils;
mod config;
mod event;
mod process;
mod dispatch;
mod collector;

use config::parse_config;
use utils::nth_prime;
use process::Process;
use dispatch::Dispatch;
use collector::{Collector, Message};

fn run_thread(process: Process) {
    info!("Thread: {}, prime: {}", process.id, process.prime);
    process.handle_dispatch()
}

fn run_dispatch(dispatch: Dispatch) {
    info!("Starting dispatcher");
    dispatch.handle_dispatch();
}

fn run_collector(collector: Collector) {
    info!("Starting collector");
    collector.handle_dispatch();
}

fn main() {
    let config = parse_config("/Users/bIgB/.config/rust-evc-log/config.toml").unwrap_or_else(|e| {
        error!("Unable to read config");
        panic!("Error: {}", e);
    });
    debug!("Initializing with config: {:?}", config);

    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(simplelog::LevelFilter::Info, simplelog::Config::default())
            .expect("Failed to create term logger"),
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            File::create("rust-evc.log").expect("Failed to create log file"),
        ),
    ]).expect("Failed to init logger");

    info!("Setting up with {} threads", config.num_processes);

    let mut thread_handles = vec![];
    let mut process_map = HashMap::new();

    let (dispatch_sender, receiver) = channel();
    let (collector_sender, collector_receiver) = channel::<Message>();

    let shared_config = Arc::new(config);

    for i in 0..shared_config.num_processes {
        let (sender, receiver) = channel();
        process_map.insert(i as u64, sender);

        let process = Process::new(
            i as u64,
            nth_prime(i as u64),
            receiver,
            dispatch_sender.clone(),
            collector_sender.clone(),
            shared_config.clone(),
        );

        thread_handles.push(spawn(move || run_thread(process)));
    }

    let dispatch_process = Dispatch::new(
        receiver,
        collector_sender.clone(),
        process_map,
        shared_config.clone(),
    );
    let collector_process = Collector::new(collector_receiver);

    thread_handles.push(spawn(move || run_dispatch(dispatch_process)));
    thread_handles.push(spawn(move || run_collector(collector_process)));

    for handle in thread_handles {
        handle.join().unwrap();
    }
}
