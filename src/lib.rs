mod histfile;

use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use walkdir::WalkDir;

use crate::histfile::{HistfileEntry, HistoryType};

fn standardise_paths(work_dir: PathBuf, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let (paths, invalid): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|f| f.is_file() || f.is_dir() || f.is_symlink());
    invalid.into_iter().for_each(|inv_path| {
        eprintln!(
            "File does not exist or is unsupported type: {}\nSkipping...",
            inv_path.to_string_lossy()
        )
    });
    paths
        .into_iter()
        .map(|p| work_dir.join(p).clean())
        .collect()
}

pub fn copy(histfile: File, paths: Vec<PathBuf>) -> Result<()> {
    let work_dir = std::env::current_dir().context("Could not stat current working directory")?;
    histfile::write_entry(
        histfile,
        HistoryType::Copy,
        standardise_paths(work_dir, paths),
    )
}

// 'move' is a keyword :)
pub fn move_cmd(histfile: File, paths: Vec<PathBuf>) -> Result<()> {
    let work_dir = std::env::current_dir().context("Could not stat current working directory")?;
    histfile::write_entry(
        histfile,
        HistoryType::Move,
        standardise_paths(work_dir, paths),
    )
}

pub fn clear(histfile: File) -> Result<()> {
    histfile.set_len(0).context("Could not truncate histfile")
}

pub fn paste(
    histfile: File,
    index: Option<usize>,
    dest: Option<PathBuf>,
    resolve_symlinks: bool,
) -> Result<()> {
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
        HistoryType::Move => entry.paths.iter().for_each(|path| {
            if let Err(err) = fs::rename(path, &(work_dir.join(path.file_name().unwrap()))) {
                eprintln!(
                    "Could not move file {} to destination {}: {err}",
                    path.to_str().unwrap(),
                    work_dir.to_str().unwrap()
                );
            };
        }),
        HistoryType::Copy => {
            for p in entry.paths {
                if p == work_dir {
                    eprintln!("Error: Cannot copy directory inside itself");
                    continue;
                }
                let base_dir = p.parent().expect("Given a one-member path");
                for entry in WalkDir::new(&p) {
                    match entry {
                        Ok(entry) => {
                            let rel_path = entry
                                .path()
                                .strip_prefix(&base_dir)
                                .context("Could not create relative path")?;
                            let dest_path = work_dir.join(rel_path);

                            if entry.file_type().is_dir() {
                                match fs::create_dir(&dest_path) {
                                    // Ok with directories existing
                                    Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                                    }
                                    Ok(_) => {}
                                    Err(e) => {
                                        bail!("Could not create directory: {}", e.to_string())
                                    }
                                }
                            } else if entry.file_type().is_symlink() && !resolve_symlinks {
                                let symlink_target = fs::read_link(&entry.path())
                                    .context("Could not resolve symlink")?;
                                std::os::unix::fs::symlink(symlink_target, dest_path)?;
                            } else if entry.file_type().is_file() {
                                fs::copy(entry.path(), &dest_path).with_context(|| {
                                    format!(
                                        "Could not copy file: {} to destination: {}",
                                        entry.path().to_string_lossy(),
                                        dest_path.to_string_lossy()
                                    )
                                })?;
                            } else {
                                eprintln!(
                                    "Invalid file type for file: {}",
                                    entry.into_path().to_string_lossy()
                                );
                            }
                        }
                        Err(err) => eprintln!("Unable to stat file: {}", err.to_string()),
                    }
                }
            }
        }
        HistoryType::Unknown => unreachable!(),
    };

    Ok(())
}

pub fn history(histfile: File, req_number: Option<usize>) -> Result<()> {
    let num_entries = req_number.unwrap_or(5);
    let entries: Vec<HistfileEntry> = histfile::get_last_n(histfile, num_entries)
        .with_context(|| format!("Could not get last {} entries in histfile", num_entries))?;
    if entries.is_empty() {
        bail!("No entries in histfile.");
    }
    println!("idx | type");
    entries.into_iter().enumerate().for_each(|(i, entry)| {
        println!("----------");
        println!("{i:>3} | {}", entry.hist_type);
        entry.paths.iter().for_each(|path| {
            println!(
                " > {}",
                match path.to_str() {
                    Some(v) => v,
                    None => "Error: could not read entry",
                }
            )
        });
    });
    Ok(())
}
