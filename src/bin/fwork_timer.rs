use std::{
    fs::File,
    io::{self, BufRead, BufReader, StderrLock, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use chrono::{DateTime, Datelike, Local, Timelike};
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

    let path = Path::new("./TIME_USAGE_A.md");
    let f = File::options()
        .read(true)
        .append(true)
        .create(true)
        .open(path);
    let mut f = match f {
        Ok(f) => f,
        Err(_) => {
            println!("File could not be written");
            return;
        }
    };

    find_last_table(&f);
    let content = {
        let date = format!("{}.{}", start.day(), start.month());
        let time = get_time_quater(&start);
        let first_cell = format!("{} {}", date, time);
        let second_cell = get_rounded_duration(duration.num_minutes());
        format!("| {} | {} | |", first_cell, second_cell)
    };
    let _ = writeln!(f, "{}", content);
}

fn find_last_table(file: &File) {
    let mut last = 0;
    let mut lines = BufReader::new(file);
    let mut line = String::new();
    let mut pos = 0;
    while let Ok(num) = lines.read_line(&mut line) {
        if (num) == 0 {
            break;
        }

        if line.starts_with("## Week ") {
            println!("{}", pos);
            print!("{}", line);
            last = pos;
        }

        pos += num;
        line.clear()
    }
}

fn get_rounded_duration(min: i64) -> String {
    let m = min;
    let h = m / 60;
    let m = nearest_quater_hour(m);

    if m % 100 == 0 {
        format!("{}h", h + m / 100)
    } else {
        format!("{}.{}h", h + m / 100, m % 100)
    }
}

fn nearest_quater_hour(min: i64) -> i64 {
    let m = (min % 60) as f64 / 60.0;
    let m = ((m + 0.125) * 4.0) as i64;
    m * 100 / 4
}

fn get_time_quater(time: &DateTime<Local>) -> String {
    let m = {
        // Get closest quter in minutes
        let mut m = time.minute() as f32;
        m += 7.5;
        let mut m = (m / 15.0) as i32 | 0;
        m *= 15;
        m %= 60;
        println!("{}", m);
        m
    };
    let h = time.hour();
    let h = if time.minute() > 52 {
        if h == 23 {
            0
        } else {
            h + 1
        }
    } else {
        h
    };

    format!("{}:{}", h, m)
}
