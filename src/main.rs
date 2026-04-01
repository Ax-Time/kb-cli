use clap::{Parser, Subcommand};

mod commands;
mod config;
mod db;
mod embed;
mod error;
mod indexer;

use config::{db_path, models_dir, resolve_kb_path};
use db::open_db;
use embed::{download_model, Embedder};
use error::Result;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long, conflicts_with = "local")]
    global: bool,

    #[arg(long, conflicts_with = "global")]
    local: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        input: String,

        #[arg(short, long)]
        recursive: bool,
    },
    Search {
        query: String,

        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    Delete {
        id: i64,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let kb_dir = resolve_kb_path(cli.local, cli.global)?;

    let models = models_dir(&kb_dir);
    let model_dir = download_model(&models)?;
    let mut embedder = Embedder::new(&model_dir)?;

    let db = db_path(&kb_dir);
    let conn = open_db(&db)?;

    match cli.command {
        Commands::Add { input, recursive } => {
            commands::add::add(&conn, &mut embedder, &input, recursive)?;
        }
        Commands::Search { query, limit } => {
            commands::search::search(&conn, &mut embedder, &query, limit)?;
        }
        Commands::Delete { id } => {
            commands::delete::delete(&conn, id)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
