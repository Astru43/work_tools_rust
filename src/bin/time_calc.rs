use clap::Parser;
use work_tools::time_usage_parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Only include latest week
    #[arg(short, long, conflicts_with("weeks"))]
    latest: bool,
    /// Show given amount of weeks
    #[arg(short = 'W', long, default_value_t)]
    weeks: i32,
    /// Delete all auto genereted files
    #[arg(short, long, exclusive(true))]
    clean: bool,
}

fn remove_file(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::path::Path::new(file_name);
    if file_path.try_exists()? {
        println!("Removing {}", file_name);
        std::fs::remove_file(file_path)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("{:#?}", args);

    if args.clean {
        remove_file("TOTAL.md")?;
        remove_file("time.csv")?;
        return Ok(());
    } else if args.latest {
        let _ = time_usage_parser::parse(std::path::Path::new("TIME_USAGE.md"));
    };

    Ok(())
}
