mod app;
mod audio;
mod config;
mod error;
mod lipsync;
mod translation;
mod video;
mod voice_synthesis;

use crate::config::Config;
use app::run;
use clap::{command, Parser};
use std::path::PathBuf;
use tracing::error;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Parser, Debug)]
#[command(
    name = "AI Video Translator",
    version = "1.0.0",
    about = "AI-assisted video translation and lip sync tool"
)]
pub struct Args {
    #[arg(short, long)]
    pub input: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(short = 's', long = "source-lang", default_value = "en-US")]
    pub source_lang: String,

    #[arg(short = 't', long = "target-lang")]
    pub target_lang: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let config = Config::load(DEFAULT_CONFIG_PATH).unwrap();
    let args = Args::parse();

    if let Err(e) = run(args, config).await {
        error!("error: {}", e);
    }
}
