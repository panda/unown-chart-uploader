use colored::*;
use reqwest::blocking::multipart;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::cli::Args;
use crate::error::{Result, UploadError};
use crate::models::{ApiResponse, ImportResult, UploadResult};

pub struct Uploader {
    client: reqwest::blocking::Client,
    upload_url: String,
    token: String,
    dry_run: bool,
    continue_on_error: bool,
    verbose: bool,
    max_retries: u32,
    retry_delay: Duration,
    base_path: PathBuf,
}

pub struct UploadResults {
    pub results: Vec<UploadResult>,
    pub success_count: usize,
    pub skip_count: usize,
    pub error_count: usize,
}

impl UploadResults {
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

impl Uploader {
    pub fn new(args: &Args) -> Self {
        // Unown IR Server chart ingest path is "/charts/import/ksh"
        let upload_url = format!("{}/charts/import/ksh", args.server.trim_end_matches('/'));

        Self {
            client: reqwest::blocking::Client::new(),
            upload_url,
            token: args.token.clone(),
            dry_run: args.dry_run,
            continue_on_error: args.continue_on_error,
            verbose: args.verbose,
            max_retries: args.max_retries,
            retry_delay: Duration::from_secs(args.retry_delay),
            base_path: args.path.clone(),
        }
    }

    pub fn upload_files(&self, files: &[PathBuf]) -> Result<UploadResults> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut skip_count = 0;
        let mut error_count = 0;

        println!("\n{}", "Starting upload...".blue());

        for (index, file_path) in files.iter().enumerate() {
            let progress = format!("[{}/{}]", index + 1, files.len());

            // Try to get relative path from base path, fall back to full path
            let display_path = file_path.strip_prefix(&self.base_path).unwrap_or(file_path);

            print!(
                "{} Uploading {}... ",
                progress.cyan(),
                display_path.display()
            );
            io::stdout().flush().unwrap();

            if self.dry_run {
                println!("{}", "SKIPPED (dry run)".yellow());
                skip_count += 1;
                continue;
            }

            let result = self.upload_file_with_retry(file_path);

            match &result {
                Ok(response) => {
                    match response.status.as_str() {
                        "imported" => {
                            println!("{}", "SUCCESS".green());
                            success_count += 1;
                        }
                        "skipped" => {
                            println!("{} (already exists)", "SKIPPED".yellow());
                            skip_count += 1;
                        }
                        _ => {
                            println!("{} ({})", "FAILED".red(), response.message);
                            error_count += 1;
                        }
                    }

                    if self.verbose {
                        self.print_verbose_info(response);
                    }

                    results.push(UploadResult {
                        path: file_path.clone(),
                        success: response.status == "imported",
                        message: response.message.clone(),
                    });
                }
                Err(e) => {
                    println!("{} ({})", "ERROR".red(), e);
                    error_count += 1;

                    results.push(UploadResult {
                        path: file_path.clone(),
                        success: false,
                        message: e.to_string(),
                    });

                    if !self.continue_on_error {
                        eprintln!("\n{}: Upload stopped due to error", "Error".red());
                        break;
                    }
                }
            }
        }

        Ok(UploadResults {
            results,
            success_count,
            skip_count,
            error_count,
        })
    }

    fn upload_file_with_retry(&self, file_path: &Path) -> Result<ImportResult> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                print!(" Retry {}/{}... ", attempt, self.max_retries);
                io::stdout().flush().unwrap();
                thread::sleep(self.retry_delay);
            }

            match self.upload_file(file_path) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        print!(" {}.", "Failed".red());
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn upload_file(&self, file_path: &Path) -> Result<ImportResult> {
        let mut file = File::open(file_path)
            .map_err(|e| UploadError::FileNotFound(format!("{}: {}", file_path.display(), e)))?;
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)?;

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("chart.ksh");

        // multipart form data
        let form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(file_content)
                .file_name(filename.to_string())
                .mime_str("application/octet-stream")
                .map_err(|e| UploadError::Other(e.to_string()))?,
        );

        let response = self
            .client
            .post(&self.upload_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()?;

        if !response.status().is_success() {
            return Err(UploadError::Other(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Parse response
        let api_response: ApiResponse<ImportResult> = response.json()?;

        // Unown IR Server returns 20x for success.
        if api_response.status_code != 20 {
            return Err(UploadError::ServerError {
                code: api_response.status_code,
                message: api_response.description,
            });
        }

        api_response
            .body
            .ok_or_else(|| UploadError::InvalidResponse("Missing body in response".to_string()))
    }

    fn print_verbose_info(&self, response: &ImportResult) {
        println!("  Title: {}", response.title);
        println!("  Artist: {}", response.artist);
        println!(
            "  Level: {} (Difficulty: {})",
            response.level, response.difficulty
        );
        println!("  Hash: {}", response.chart_hash);
        println!("  Message: {}", response.message);
    }
}
