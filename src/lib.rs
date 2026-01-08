mod histfile;

use anyhow::{Context, Result, bail};
use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::histfile::{HistfileEntry, HistoryType};

pub fn copy(histfile: File, paths: Vec<PathBuf>) -> Result<()> {
    histfile::write_entry(histfile, HistoryType::Copy, paths)
}

// 'move' is a keyword :)
pub fn do_move(histfile: File, paths: Vec<PathBuf>) -> Result<()> {
    histfile::write_entry(histfile, HistoryType::Move, paths)
}

pub fn paste(histfile: File, index: Option<usize>, dest: Option<PathBuf>) -> Result<()> {
    let mut work_dir = std::env::current_dir()
        .ok()
        .context("Could not retreive working directory from env.")?;

    // Overwrites if provided path is absolute
    if let Some(user_dir) = dest {
        work_dir = work_dir.join(user_dir);
    }

    if work_dir.is_file() {
        bail!("User provided file instead of directory.")
    }

    let entry = histfile::read_entry(histfile, index.unwrap_or(0))?;
    match entry.hist_type {
        HistoryType::Copy => entry.paths.iter().for_each(|path| {
            if let Err(err) = fs::copy(path, &(work_dir.join(path.file_name().unwrap()))) {
                eprintln!(
                    "Could not copy file {} to destination {}: {err}",
                    path.to_str().unwrap(),
                    work_dir.to_str().unwrap()
                );
            };
        }),
        HistoryType::Move => entry.paths.iter().for_each(|path| {
            if let Err(err) = fs::copy(path, &(work_dir.join(path.file_name().unwrap()))) {
                eprintln!(
                    "Could not move file {} to destination {}: {err}",
                    path.to_str().unwrap(),
                    work_dir.to_str().unwrap()
                );
            };
        }),
    };

    Ok(())
}

pub fn history(histfile: File, req_number: Option<usize>) -> Result<()> {
    println!("Getting history");
    let num_entries = req_number.unwrap_or(5);
    let entries: Vec<HistfileEntry> = histfile::get_last_n(histfile, num_entries)
        .with_context(|| format!("Could not get last {} entries in histfile", num_entries))?;
    println!("idx|type");
    entries.into_iter().enumerate().for_each(|(i, entry)| {
        println!("({i}) {}", entry.hist_type);
        entry.paths.iter().for_each(|path| {
            println!(
                "\t> {}",
                match path.to_str() {
                    Some(v) => v,
                    None => "Error: could not read entry",
                }
            )
        });
    });
    Ok(())
}
