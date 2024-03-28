use std::fmt::Write as _;
use std::fs::File;
use std::io::Write as _;
use std::{io, path::Path};

use ansi_term::Color;
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
    /// Generate csv of selected weeks called time.csv
    #[arg(long)]
    csv: bool,
    /// File
    #[arg(default_value("TIME_USAGE.md"))]
    file: String,
}

fn alternate<'a>(string: &'a str, odd: &'a bool) -> ansi_term::ANSIGenericString<'a, str> {
    if *odd {
        return ansi_term::ANSIString::from(string);
    }

    ansi_term::Style::new().dimmed().paint(string)
}

fn week_total(week: &Week) -> f32 {
    let mut result = 0.0;
    for day in &week.days {
        for hour in &day.hours {
            result += hour.duration;
        }
    }
    result
}

fn print_week(week: &Week, week_total: &f32) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut handle = io::BufWriter::new(lock);

    let header_color = Color::Yellow.bold();
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

    let total_string = Color::Cyan
        .bold()
        .paint(format!("Total\t\t{}h", week_total));
    writeln!(handle, "{}", total_string)?;
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
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();
    let args = Cli::parse();

    // let time_usage_path = Path::new::<str>(args.file.as_ref());
    let time_usage_path = Path::new::<String>(&args.file);
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
                let _ = print_week(&week, &week_total(&week));
                if args.csv {
                    let _ = write_csv(&weeks[weeks.len() - 1..weeks.len()]);
                }
                Ok(())
            })?;
        }

        Cli { weeks: count, .. } if count > 0 => {
            with_weeks(time_usage_path, |weeks: Vec<Week>| {
                let Some(start) = weeks.len().checked_sub(count.into()) else {
                    return Err(anyhow!("Index out of bounds, max len: {}", weeks.len()));
                };
                let mut cycel_total = 0.0;
                for week in &weeks[start..weeks.len()] {
                    let week_total = week_total(week);
                    cycel_total += week_total;
                    let _ = print_week(week, &week_total);
                }

                println!("Cycle total\t{}h", cycel_total);
                if args.csv {
                    let _ = write_csv(&weeks[start..weeks.len()]);
                }
                Ok(())
            })?;
        }
        _ => {
            with_weeks(time_usage_path, |weeks| {
                let mut total = 0.0;
                for week in &weeks {
                    let w_total = week_total(&week);
                    total += w_total;
                    let _ = print_week(&week, &w_total);
                }
                println!(
                    "{}",
                    Color::Cyan.bold().paint(format!("Total\t\t{}h", total))
                );
                if args.csv {
                    let _ = write_csv(&weeks);
                }
                Ok(())
            })?;
        }
    }

    Ok(())
}

fn write_csv(weeks: &[Week]) -> anyhow::Result<()> {
    let mut file = File::create("time.csv")?;

    let mut output = String::new();
    for week in weeks {
        output += &(week.week_start.clone() + "\n");
        let mut total = 0.0;
        for day in &week.days {
            for hours in &day.hours {
                total += hours.duration;
                let line = format!(
                    "{} {},\"{}\",\"{}\"\n",
                    day.date,
                    hours.time,
                    hours.duration.to_string().replace(".", ","),
                    hours.task
                );
                output += &line;
            }
        }
        output += &format!("Total,\"{}\"\n\n", total.to_string().replace(".", ","));
    }
    write!(file, "{}", output)?;

    Ok(())
}
