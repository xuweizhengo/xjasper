//! XJasper CLI
//!
//! Command-line interface for XJasper.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use xjasper_engine::ReportEngine;

#[derive(Parser, Debug)]
#[command(name = "xjasper")]
#[command(about = "XJasper Report Generator", long_about = None)]
struct Args {
    /// Path to the template JSON file
    #[arg(short, long)]
    template: PathBuf,

    /// Path to the data JSON file
    #[arg(short, long)]
    data: PathBuf,

    /// Path to the output PDF file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Read template file
    let template_json = fs::read_to_string(&args.template)?;

    // Read data file
    let data_json = fs::read_to_string(&args.data)?;

    // Create engine and render
    let mut engine = ReportEngine::new();
    let pdf_bytes = engine.render(&template_json, &data_json)?;

    // Write output
    fs::write(&args.output, pdf_bytes)?;

    println!("✓ Report generated: {}", args.output.display());

    Ok(())
}

