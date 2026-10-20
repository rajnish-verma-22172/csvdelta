use clap::{Parser, ValueEnum};
use csv::StringRecord;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use csvdelta::{compare, output, DiffReport};

#[derive(Debug, Parser)]
#[command(version, about = "Show what changed between two CSV files")]
struct Args {
    old: PathBuf,
    new: PathBuf,
    #[arg(long)]
    key: Option<String>,
    #[arg(long, value_enum, default_value_t = Format::Table)]
    format: Format,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Table,
    Json,
    Markdown,
}

struct CsvData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn main() -> ExitCode {
    match run() {
        Ok(has_changes) => {
            if has_changes {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("csvdelta: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let old = read_csv(&args.old)?;
    let new = read_csv(&args.new)?;
    let key = args
        .key
        .or_else(|| old.headers.first().cloned())
        .ok_or("old CSV file has no header row")?;
    let report = compare(&old.headers, &old.rows, &new.headers, &new.rows, &key)?;
    print_report(&report, args.format)?;
    Ok(report.has_changes())
}

fn read_csv(path: &Path) -> Result<CsvData, Box<dyn std::error::Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .from_path(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(|error| format!("{}: {error}", path.display()))?
        .iter()
        .map(str::to_owned)
        .collect();
    if headers.is_empty() || headers.iter().all(String::is_empty) {
        return Err(format!("{} is empty or has no header row", path.display()).into());
    }
    let mut rows = Vec::new();
    for record in reader.records() {
        let record: StringRecord =
            record.map_err(|error| format!("{}: {error}", path.display()))?;
        rows.push(record.iter().map(str::to_owned).collect());
    }
    Ok(CsvData { headers, rows })
}

fn print_report(report: &DiffReport, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        Format::Table => print!("{}", output::render_table(report)),
        Format::Json => println!("{}", output::render_json(report)?),
        Format::Markdown => print!("{}", output::render_markdown(report)),
    }
    Ok(())
}
