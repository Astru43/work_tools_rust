use std::path::Path;

use clap::Parser;
use work_tools::time_usage_parser::parse;

#[derive(Debug, Parser)]
struct Cli {
    /// Only include latest week
    #[arg(short, long, conflicts_with("weeks"))]
    latest: bool,
    /// Show given amount of weeks
    #[arg(short = 'W', long, default_value_t)]
    weeks: i32,
    /// Delete all auto genereted files in current directory
    #[arg(short, long, exclusive(true))]
    clean: bool,
}

fn remove_file(path: &Path) -> anyhow::Result<()> {
    if path.try_exists()? {
        println!("Removing {}", path.display());
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    println!("{:#?}", args);

    if args.clean {
        remove_file(Path::new("TOTAL.md"))?;
        remove_file(Path::new("time.csv"))?;
        return Ok(());
    } else if args.latest {
        let weeks = parse(Path::new("TIME_USAGE.md"))?;
        match weeks {
            Some(weeks) => {
                for week in weeks {
                    println!("{}", week.week_start)
                }
            }
            None => (),
        }
    };

    Ok(())
}
