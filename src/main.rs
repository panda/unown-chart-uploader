mod cli;
mod error;
mod models;
mod upload;
mod utils;

use colored::*;
use std::io::{self, Write};
use std::time::Instant;

use crate::error::Result;
use crate::upload::Uploader;
use crate::utils::find_ksh_files;

fn main() -> Result<()> {
    let args = cli::parse_args()?;

    if !args.path.exists() {
        eprintln!(
            "{}: Path '{}' does not exist",
            "Error".red(),
            args.path.display()
        );
        std::process::exit(1);
    }

    let ksh_files = find_ksh_files(&args.path)?;

    if ksh_files.is_empty() {
        println!(
            "{}: No .ksh files found in '{}'",
            "Warning".yellow(),
            args.path.display()
        );
        return Ok(());
    }

    println!(
        "{} Found {} .ksh files",
        "Info".blue(),
        ksh_files.len().to_string().green()
    );

    if args.verbose {
        print_files_list(&ksh_files);
    }

    // Confirm upload unless --yes flag is provided or dry run
    if !args.yes && !args.dry_run && !confirm_upload(&ksh_files, &args.server)? {
        println!("{}", "Upload cancelled".yellow());
        return Ok(());
    }

    // Create uploader and perform uploads
    let uploader = Uploader::new(&args);
    let start_time = Instant::now();
    let results = uploader.upload_files(&ksh_files)?;

    print_summary(
        &results,
        ksh_files.len(),
        start_time.elapsed(),
        args.verbose,
    );

    // Exit with appropriate code
    let exit_code = (results.has_errors() && !args.continue_on_error) as i32;

    std::process::exit(exit_code);
}

fn print_files_list(files: &[std::path::PathBuf]) {
    println!("\n{}", "Files to upload:".blue());
    for file in files {
        println!("  - {}", file.display());
    }
}

fn confirm_upload(files: &[std::path::PathBuf], server: &str) -> Result<bool> {
    println!(
        "\n{} Upload {} files to {}?",
        "Confirm".yellow(),
        files.len(),
        server.green()
    );
    print!("Continue? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn print_summary(
    results: &upload::UploadResults,
    total_files: usize,
    elapsed: std::time::Duration,
    verbose: bool,
) {
    println!("\n{}", "=== Upload Summary ===".blue());
    println!("Total files: {}", total_files);
    println!("Successful: {}", results.success_count.to_string().green());
    println!("Skipped: {}", results.skip_count.to_string().yellow());
    println!("Errors: {}", results.error_count.to_string().red());
    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());

    if results.error_count > 0 && verbose {
        println!("\n{}", "Failed uploads:".red());
        for result in &results.results {
            if !result.success {
                println!("  - {}: {}", result.path.display(), result.message);
            }
        }
    }
}
