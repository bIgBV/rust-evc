use config::Config;
use event::Event;
use rug::float::Round;
use rug::Float;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use rayon::prelude::*;
use std::mem::size_of_val;

#[derive(Debug)]
pub struct Collector {
    pub events: Vec<Event>,
    receiver: Receiver<Message>,
    config: Arc<Config>,
    end_counter: i64,
    event_pairs: Vec<Pair>,
}

#[derive(Debug)]
struct Metrics {
    false_negatives: u32,
    false_positives: u32,
    true_negatives: u32,
    true_positives: u32,
}

impl Collector {
    pub fn new(receiver: Receiver<Message>, config: Arc<Config>) -> Collector {
        Collector {
            events: Vec::new(),
            receiver,
            config,
            end_counter: 0,
            event_pairs: Vec::new(),
        }
    }

    pub fn handle_dispatch(mut self) {
        for message in self.receiver {
            match message {
                Message::Data(e) => {
                    let evc_size = &e.encoded_clock.significant_bits();
                    self.events.push(e);

                    // if self.events.len() % 2 == 0 {
                    //     error!(
                    //         "{}, {}",
                    //         self.events.len(),
                    //         evc_size
                    //     );
                    // }
                }
                Message::End => {
                    self.end_counter += 1;

                    if self.end_counter == self.config.num_processes + 1 {
                        //error!(
                        //    "Processes: {} Events: {}.",
                        //    self.config.num_processes,
                        //    self.events.len()
                        //);

                        //let false_positives =
                        //    count_false_positives(self.config.float_precision, &self.event_pairs);
                        //error!(
                        //    "Total pairs: {} false positives: {}",
                        //    self.event_pairs.len(),
                        //    false_positives
                        //);

                        let event_pairs = generate_permutations(&self.events);
                        let length = event_pairs.len();
                        // let (false_negatives, false_positives) =
                        //     check_log_clocks(event_pairs, self.config.float_precision);

                        let false_positives =
                            check_log_clocks_fp(event_pairs, self.config.float_precision);

                        error!(
                            "{}, {}, {}, {}",
                            self.config.num_processes,
                            self.config.float_precision,
                            self.config.max_bits,
                            (false_positives as f64 / length as f64) * 100_f64
                        );

                        break;
                    }
                }
                Message::Pair(p) => self.event_pairs.push(p),
            }
        }
    }
}

fn check_log_clocks(pairs: Vec<Pair>, prec: u32) -> (u32, u32) {
    let (false_negatives, false_positives) = pairs
        .iter()
        .map(|pair| count_false_values(&pair, prec))
        .fold((0, 0), |acc: (u32, u32), count: (u32, u32)| {
            (acc.0 + count.0, acc.1 + count.1)
        });

    (false_negatives, false_positives)
}

fn check_log_clocks_fp(pairs: Vec<Pair>, prec: u32) -> Metrics {
    let mut final_counts = Metrics {
        false_negatives: 0,
        false_positives: 0,
        true_negatives: 0,
        true_positives: 0,
    };

    let false_positives = pairs
        .iter()
        .map(|pair| count_false_positives(&pair, prec))
        .fold(final_counts, |acc: Metrics, metric: Metrics| {
            Metrics {
                false_positives: acc.false_positives + metric.false_positives,
                false_negatives: acc.false_negatives + metric.false_negatives,
                true_negatives: acc.true_negatives + metric.true_negatives,
                true_positives: acc.true_positives + metric.true_positives,
            }
        });

    final_counts
}

fn count_false_positives(pair: &Pair, prec: u32) -> Metrics {
    let mut false_positives: u32 = 0;
    let mut false_negatives: u32 = 0;
    let mut true_negatives: u32 = 0;
    let mut true_positives: u32 = 0;

    let log_send = Float::with_val(prec, &pair.send.encoded_clock).log2();
    let log_recv = Float::with_val(prec, &pair.receive.encoded_clock).log2();
    let mut log_diff_left = Float::with_val(prec, &log_recv - &log_send);
    let mut log_diff_right = Float::with_val(prec, &log_send - &log_recv);
    log_diff_left.exp2_round(Round::Nearest);
    log_diff_right.exp2_round(Round::Nearest);

    if pair.send
        .encoded_clock
        .is_divisible(&pair.receive.encoded_clock)
        && pair.receive
            .encoded_clock
            .is_divisible(&pair.send.encoded_clock)
    {
        if log_diff_left.is_integer() || log_diff_right.is_integer() {
            false_positives += 1;
        }

        if log_diff_left.is_integer() && log_diff_right.is_integer() {
            true_negatives += 1;
        }
    }

    if (pair.receive
        .encoded_clock
        .is_divisible(&pair.send.encoded_clock) && log_diff_left.is_integer())
        || (pair.send
            .encoded_clock
            .is_divisible(&pair.receive.encoded_clock) && log_diff_right.is_integer())
    {
        true_positives += 1
    }

    if (pair.receive
        .encoded_clock
        .is_divisible(&pair.send.encoded_clock) && !log_diff_left.is_integer())
        || (pair.send
            .encoded_clock
            .is_divisible(&pair.receive.encoded_clock) && !log_diff_right.is_integer())
    {
        false_negatives += 1;
    }

    Metrics {
        false_negatives: false_negatives,
        false_positives: false_positives,
        true_negatives: true_negatives,
        true_positives: true_positives,
    }
}

fn count_false_values(pair: &Pair, prec: u32) -> (u32, u32) {
    let mut false_negatives: u32 = 0;
    let mut false_positives: u32 = 0;

    let log_send = Float::with_val(prec, &pair.send.encoded_clock).log2();
    let log_recv = Float::with_val(prec, &pair.receive.encoded_clock).log2();

    if less_than(&pair.send, &pair.receive) {
        let mut log_diff = Float::with_val(prec, &log_recv - &log_send);
        log_diff.exp2_round(Round::Up);

        if log_send < log_recv && log_diff.is_integer() {
            false_negatives += 0;
        } else {
            false_negatives += 1;
        }
    } else if less_than(&pair.receive, &pair.send) {
        let mut log_diff = Float::with_val(prec, &log_send - &log_recv);
        log_diff.exp2_round(Round::Up);

        if log_recv < log_send && log_diff.is_integer() {
            false_negatives += 0;
        } else {
            false_negatives += 1;
        }
    } else if !less_than(&pair.send, &pair.receive) {
        let mut log_diff = Float::with_val(prec, &log_recv - &log_send);
        log_diff.exp2_round(Round::Up);

        if log_send < log_recv && log_diff.is_integer() {
            false_positives += 1;
        }
    } else if !less_than(&pair.receive, &pair.send) {
        let mut log_diff = Float::with_val(prec, &log_send - &log_recv);
        log_diff.exp2_round(Round::Up);

        if log_recv < log_send && log_diff.is_integer() {
            false_positives += 1;
        }
    }

    (false_negatives, false_positives)
}

fn less_than(first: &Event, other: &Event) -> bool {
    first.vec_clock[first.process_id as usize] <= (other.vec_clock[first.process_id as usize])
}

fn generate_permutations(events: &[Event]) -> Vec<Pair> {
    let mut result_vec = Vec::new();
    let len = events.len();

    for (i, event_i) in events.into_iter().enumerate() {
        if i < len {
            for event_j in events[i + 1..].into_iter() {
                result_vec.push(event_i.clone().make_pair(event_j.clone()));
            }
        }
    }

    result_vec
}

pub trait ToPair {
    fn make_pair(self, other: Event) -> Pair;
}

#[derive(Debug, Clone)]
pub struct Pair {
    pub send: Event,
    pub receive: Event,
}

#[derive(Debug, Clone)]
pub enum Message {
    Data(Event),
    Pair(Pair),
    End,
}
