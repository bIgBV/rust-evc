use std::thread::spawn;
use std::fs::File;
use std::sync::mpsc::channel;
use std::collections::HashMap;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;

mod utils;
mod config;
mod event;
mod process;

use config::parse_config;
use utils::nth_prime;
use process::Process;

fn run_thread(mut process: Process) {
    info!("Thread: {}, prime: {}", process.id, process.prime);
    process.handle_dispatch()
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
    let mut proces_map = HashMap::new();

    for i in 1..config.num_processes + 1 {
        let (sender, receiver) = channel();
        proces_map.insert(i, sender);
        let process = Process::new(i as u64, nth_prime(i as u64), receiver);

        thread_handles.push(spawn(move || run_thread(process)));
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}
