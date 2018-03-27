## Rust EVC

This is an implementation for the paper "Encoded Vector Clock: Using Primes to
Characterize Causality in Distributed Systems" by Ajay D. Kshemkalyani,
Ashfaq A. Khokhar, Min Shen. The application is multi-threaded with
various worker threads and a dispatcher thread counting all the events
occurring in the system as well as dispatching send events randomly between
the various threads.

Each thread simulates a single process in a distributed system. It waits for a
receive event on it's receiver end of the channel (here the channel is used as a
FIFO thread safe queue) for a configurabele amount of time after which it
will either performa an internal event or send a message to be dispatched.

The dispatcher in turn selects a random process and dispatches the message
across.
