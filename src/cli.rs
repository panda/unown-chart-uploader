use clap::Parser;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Parser, Debug)]
#[command(name = "unown-chart-uploader")]
#[command(about = "Upload KSH chart files to USC-IR server", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: PathBuf,

    #[arg(short, long)]
    pub server: String,

    #[arg(short, long)]
    pub token: String,

    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    #[arg(short = 'y', long, default_value_t = false)]
    pub yes: bool,

    #[arg(short, long, default_value_t = false)]
    pub continue_on_error: bool,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[arg(short = 'r', long, default_value_t = 3)]
    pub max_retries: u32,

    #[arg(long, default_value_t = 1)]
    pub retry_delay: u64,
}

pub fn parse_args() -> Result<Args> {
    Ok(Args::parse())
}
