use std::{
    io::{self, StderrLock, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use chrono::{DateTime, Local};
use hhmmss::Hhmmss;

macro_rules! fmt {
    () => {
        "{:12}{}"
    };
    ($arg:literal) => {
        concat!($arg, fmt!())
    };
}

fn print_status(start: &DateTime<Local>, lock: &mut StderrLock) {
    let duration = Local::now() - start;
    write!(lock, fmt!('\r'), "Duration:", duration.hhmmss()).unwrap();
    io::stderr().flush().unwrap();
}

fn main() {
    let quit = Arc::new(AtomicBool::new(false));
    let q = quit.clone();
    ctrlc::set_handler(move || q.store(true, Ordering::SeqCst))
        .expect("Error setting ctrl + c handler");

    let start = Local::now();
    println!(fmt!(), "Start:", start);

    let mut lock = io::stderr().lock();
    while !quit.load(Ordering::SeqCst) {
        print_status(&start, &mut lock);
        sleep(Duration::from_secs(1));
    }

    let end = Local::now();
    println!(fmt!('\r'), "End:", end);

    let duration = end - start;
    println!(fmt!(), "Duration:", duration.hhmmss());
}
