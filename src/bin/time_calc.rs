use std::fmt::Write as _;
use std::io::Write as _;
use std::{io, path::Path};

use anyhow::anyhow;
use clap::Parser;
use work_tools::time_usage_parser::{parse, Week};

#[derive(Debug, Parser)]
struct Cli {
    /// Only include latest week
    #[arg(short, long, conflicts_with("weeks"))]
    latest: bool,
    /// Show given amount of weeks
    #[arg(short = 'W', long, default_value_t)]
    weeks: u16,
    /// Delete all auto genereted files in current directory
    #[arg(short, long, exclusive(true))]
    clean: bool,
}

fn alternate<'a>(string: &'a str, odd: &'a bool) -> ansi_term::ANSIGenericString<'a, str> {
    if *odd {
        return ansi_term::ANSIString::from(string);
    }

    ansi_term::Style::new().dimmed().paint(string)
}

fn print_week(week: &Week) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut handle = io::BufWriter::new(lock);

    let header_color = ansi_term::Color::Yellow.bold();
    let header = header_color.paint(week.week_start.clone());
    writeln!(handle, "{}", header)?;

    let mut odd = false;
    for day in &week.days {
        let date = alternate(&day.date, &odd);
        write!(handle, "{}", date)?;
        for hours in &day.hours {
            let mut line = String::new();
            write!(line, "\t{}\t{}\t{}", hours.time, hours.duration, hours.task)?;
            let line = alternate(&line, &odd);
            writeln!(handle, "{}", line)?;
        }
        odd = !odd;
    }

    writeln!(handle)?;
    Ok(())
}

fn with_weeks(
    path: &Path,
    callback: impl Fn(Vec<Week>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let weeks = parse(path)?;
    if let Some(weeks) = weeks {
        callback(weeks)?;
    }

    Ok(())
}

fn remove_file(path: &Path) -> anyhow::Result<()> {
    if path.try_exists()? {
        println!("Removing {}", path.display());
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let _ = ansi_term::enable_ansi_support();
    let args = Cli::parse();
    // println!("{:#?}", args);

    let time_usage_path = Path::new("TIME_USAGE.md");

    match args {
        Cli { clean: true, .. } => {
            remove_file(Path::new("TOTAL.md"))?;
            remove_file(Path::new("time.csv"))?;
        }

        Cli { latest: true, .. } => {
            with_weeks(time_usage_path, |weeks| {
                let Some(week) = weeks.last() else {
                    return Ok(());
                };
                let _ = print_week(&week);
                Ok(())
            })?;
        }

        Cli { weeks: count, .. } if count > 0 => {
            with_weeks(time_usage_path, |weeks: Vec<Week>| {
                let Some(start) = weeks.len().checked_sub(count.into()) else {
                    return Err(anyhow!("Index out of bounds, max len: {}", weeks.len()));
                };
                for week in &weeks[start..weeks.len()] {
                    let _ = print_week(week);
                }
                Ok(())
            })?;
        }
        _ => {
            with_weeks(time_usage_path, |weeks| {
                for week in weeks {
                    let _ = print_week(&week);
                }
                Ok(())
            })?;
        }
    }

    Ok(())
}
