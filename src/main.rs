#[cfg(windows)]
compile_error!(
    "Windows hath given you the Mouse and the File Explorer, use them with grace. (This crate is not supported on Windows)"
);

use std::{fs::OpenOptions, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use lazycp::{HistoryType, histfile_contents, histfile_entry};

/// The easiest way to copy for people who have too many terminals open.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Storage location for copy history
    #[arg(long, global = true, env = "LAZYCP_HISTFILE", value_parser = clap::value_parser!(PathBuf))]
    histfile: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Insert a list of files and directories into copy history for later pasting
    #[command(aliases = &["cp", "c"])]
    Copy {
        #[arg(value_parser = clap::value_parser!(PathBuf))]
        files: Vec<PathBuf>,
    },
    /// Insert a list of files or directories into move history for later pasting
    #[command(aliases = &["mv", "m"])]
    Move {
        #[arg(value_parser = clap::value_parser!(PathBuf))]
        files: Vec<PathBuf>,
    },
    /// Paste latest or specified copied files or diretories into the current directory
    #[command(alias = "p")]
    Paste {
        /// Destination relative to current directory. A bit silly, but the option is there
        #[arg(long, short, value_parser = clap::value_parser!(PathBuf))]
        dest: Option<PathBuf>,
        /// Advanced usecase - provide a host to copy from over SSH. Provide an alternative
        /// --histfile if you did not copy with the same user as provided in the host string
        #[arg(long, short = 't')]
        host: Option<String>,
        /// Resolves symlinks to the original file when provided
        #[arg(long, short)]
        resolve_symlinks: Option<bool>,
        /// Index of item within copy/move history to paste
        #[arg(long, short)]
        index: Option<usize>,
    },
    /// Show latest copy/move history
    #[command(aliases = &["hist", "h"])]
    History {
        /// Number of history entries to show
        #[arg(short)]
        number: Option<usize>,
    },
    /// Clear copy/move history for default or specified history file
    #[command(aliases = &["clr", "cl"])]
    Clear,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let histfile = args
        .histfile
        .or({
            if let Ok(cache) = std::env::var("XDG_CACHE_HOME") {
                Some(PathBuf::from(format!("{cache}/lazycp/histfile")))
            } else {
                None
            }
        })
        .or({
            if let Ok(home) = std::env::var("HOME") {
                Some(PathBuf::from(format!("{home}/.cache/lazycp/histfile")))
            } else {
                None
            }
        })
        .context("Could not find valid $XDG_CACHE_HOME or $HOME, please provide a valid --histfile location.")?;
    std::fs::create_dir_all(&histfile.parent().unwrap())
        .context("Could not create parent directories when writing histfile.")?;

    let histfile = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&histfile)
        .context("Could not create or open histfile for appending.")?;

    match args.command {
        Command::Copy { files } => histfile_entry(histfile, HistoryType::Copy, files),
        Command::Move { files } => histfile_entry(histfile, HistoryType::Move, files),
        Command::Paste {
            dest,
            host,
            resolve_symlinks,
            index,
        } => histfile_contents(histfile),
        Command::Clear => todo!(),
        Command::History { number } => todo!(),
    }
}
