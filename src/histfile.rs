use anyhow::{Context, Result, bail};
use reverse_lines::ReverseLines;
use std::{
    fmt,
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

#[derive(PartialEq, Eq)]
pub enum HistoryType {
    Move,
    Copy,
    Unknown,
}

impl fmt::Display for HistoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Move => "MOVE",
            Self::Copy => "COPY",
            _ => return Err(std::fmt::Error),
        };
        write!(f, "{text}")
    }
}

impl From<&str> for HistoryType {
    fn from(value: &str) -> Self {
        match value {
            "COPY" => Self::Copy,
            "MOVE" => Self::Move,
            _ => Self::Unknown,
        }
    }
}

pub struct HistfileEntry {
    pub hist_type: HistoryType,
    pub paths: Vec<PathBuf>,
}

pub fn write_entry(mut histfile: File, hist_type: HistoryType, paths: Vec<PathBuf>) -> Result<()> {
    if paths.len() == 0 {
        bail!("No valid paths provided");
    }

    writeln!(
        histfile,
        "{hist_type}\0{}",
        paths
            .iter()
            .map(|path| path.as_os_str())
            .collect::<Vec<_>>()
            .join(&std::ffi::OsStr::new("\0"))
            .into_string()
            .unwrap()
    )
    .context("Could not create entry in histfile. Aborting...")?;
    Ok(())
}

pub fn read_entry(histfile: File, index: usize) -> Result<HistfileEntry> {
    let rev_lines = ReverseLines::new(BufReader::new(histfile))
        .context("Could not seek or read histfile contents")?;

    let requested_line = match rev_lines.into_iter().nth(index) {
        Some(line) => line,
        None => bail!("Received invalid histfile index {}. Hint: use `history` subcommand to view existing entries", index),
    }
    .context("Could not seek or read histfile contents")?;

    entry_from_line(requested_line)
}

fn entry_from_line(line: String) -> Result<HistfileEntry> {
    let line_split: Vec<&str> = line.split("\0").into_iter().collect();

    assert!(line_split.len() >= 2);
    // SAFETY: above assert
    let hist_type_str = *(unsafe { line_split.get_unchecked(0) });
    let hist_type = HistoryType::from(hist_type_str);
    if hist_type == HistoryType::Unknown {
        bail!("Unknown history type: {}", hist_type_str);
    }

    let paths: Vec<PathBuf> = line_split[1..].iter().copied().map(PathBuf::from).collect();

    Ok(HistfileEntry { hist_type, paths })
}

pub fn get_last_n(histfile: File, num: usize) -> Result<Vec<HistfileEntry>> {
    let rev_lines = ReverseLines::new(BufReader::new(histfile))
        .context("Could not seek or read histfile contents")?;

    // Get lines and map to entry
    let mut error_count = 0;
    let mut requested_lines = vec![];
    for r in rev_lines.into_iter().take(num) {
        let line = match r {
            Ok(v) => v,
            Err(_) => {
                error_count += 1;
                continue;
            }
        };

        match entry_from_line(line) {
            Ok(entry) => requested_lines.push(entry),
            Err(_) => error_count += 1,
        }
    }

    if error_count != 0 {
        eprintln!("Warning: some history entries could not be read.");
    }

    Ok(requested_lines)
}
