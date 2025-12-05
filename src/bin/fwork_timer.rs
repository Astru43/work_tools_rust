use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, StderrLock, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use fancy_regex::Regex;
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

    let last = find_last_table(&f);
    let mut tmp = path.file_name().unwrap().to_owned();
    tmp.push("~");
    let tmp_f = File::create(path.with_file_name(tmp));
    let mut tmp_f = match tmp_f {
        Ok(f) => f,
        Err(_) => {
            println!("File could not be written");
            return;
        }
    };

    f.rewind().unwrap();
    io_copy(&mut f, &mut tmp_f, last as usize);

    let mut last_table = get_last_table(&f, last).unwrap();
    let date = Regex::new(r"- *(\d+?\.\d+?(\.\d+?)?)$").unwrap();
    let res = date.captures(&last_table[0]).unwrap();
    if let Some(res) = res {
        let last_date = res.get(1).unwrap().as_str();
        if res.get(2).is_none() {
            let last_date = NaiveDate::parse_from_str(
                format!("{last_date}.{}", start.year()).as_str(),
                "%d.%m.%Y",
            )
            .unwrap();
            println!("{last_date}");
            let date = start.date_naive();
            println!("{date}");

            println!("Last date already past");
        }
    }

    let content = {
        let date = format!("{}.{}", start.day(), start.month());
        let time = get_time_quater(&start);
        let first_cell = format!("{} {}", date, time);
        let second_cell = get_rounded_duration(duration.num_minutes());
        format!("| {} | {} | |", first_cell, second_cell)
    };

    let last_row = last_table.iter().rposition(|r| r.contains("|")).unwrap() + 1;
    println!("{}", &last_row);
    last_table.insert(last_row, content);
    let _ = println!("{}", &last_table[last_row]);

    let mut writer = BufWriter::new(tmp_f);
    write_table(&mut writer, &last_table);
}

macro_rules! KB {
    ($x: expr) => {
        $x << 10
    };
}

fn write_table(writer: &mut dyn std::io::Write, table: &Vec<String>) {
    for l in table {
        writeln!(writer, "{l}").unwrap();
    }
}

fn io_copy(reader: &mut dyn std::io::Read, writer: &mut dyn std::io::Write, count: usize) {
    let mut tmp_buf: [u8; KB!(16)] = [0; KB!(16)];
    let mut remaining = count;
    while remaining > 0 {
        let to_copy = if remaining > KB!(16) {
            KB!(16)
        } else {
            remaining
        };

        reader.read_exact(&mut tmp_buf[0..to_copy]).unwrap();
        remaining -= to_copy;
        writer.write_all(&mut tmp_buf[0..to_copy]).unwrap();
    }
}

fn get_last_table(file: &File, pos: u64) -> anyhow::Result<Vec<String>> {
    let mut lines = BufReader::new(file);
    lines.seek(SeekFrom::Start(pos))?;
    let table = lines.lines().map(|l| l.unwrap()).collect::<Vec<_>>();

    // println!("Table:\n{:?}", &table);
    Ok(table)
}

fn find_last_table(file: &File) -> u64 {
    let mut last = 0;
    let mut lines = BufReader::new(file);
    let mut line = String::new();
    while let Ok(num) = lines.read_line(&mut line) {
        if num == 0 {
            break;
        }

        if line.starts_with("## Week ") {
            last = lines
                .stream_position()
                .expect("Failed to get current position");
            last -= num as u64;
            println!("{}", last);
            println!("{}", line);
        }

        line.clear()
    }

    println!("{}", last);
    last
}

fn get_rounded_duration(min: i64) -> String {
    let m = min;
    let h = m / 60;
    let m = nearest_quater_hour(m);

    if h == 0 && m % 100 == 0 {
        String::from("0.25h")
    } else if m % 100 == 0 {
        format!("{}h", h + m / 100)
    } else {
        format!("{}.{}h", h + m / 100, m % 100)
    }
}

fn nearest_quater_hour(min: i64) -> i64 {
    let m = (min % 60) as f64 / 60.0;
    let m = ((m + 0.24) * 4.0) as i64;
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
