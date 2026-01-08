use anyhow::{Context, Result, bail};
use reverse_lines::ReverseLines;
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

pub enum HistoryType {
    Move,
    Copy,
}

impl fmt::Display for HistoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Move => "mv",
            Self::Copy => "cp",
        };
        write!(f, "{text}")
    }
}

pub struct HistfileEntry {
    pub hist_type: HistoryType,
    pub paths: Vec<PathBuf>,
}

pub fn write_entry(mut histfile: File, hist_type: HistoryType, paths: Vec<PathBuf>) -> Result<()> {
    let work_dir = std::env::current_dir().context("Could not stat current working directory")?;
    let full_paths = paths
        .into_iter()
        .map(|path| work_dir.join(path))
        .collect::<Vec<PathBuf>>();

    writeln!(
        histfile,
        "{hist_type}\0{}",
        full_paths
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
    let hist_type = match *(unsafe { line_split.get_unchecked(0) }) {
        "mv" => HistoryType::Move,
        "cp" => HistoryType::Copy,
        unknown => bail!("Invalid history type '{unknown}'"),
    };

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
    // Not enough entries
    if error_count + requested_lines.len() != num {
        eprintln!("Requested {} entries, found {}", num, requested_lines.len());
    }

    Ok(requested_lines)
}

pub fn histfile_contents(mut histfile: File) -> Result<()> {
    let mut buf = String::with_capacity(histfile.metadata().unwrap().len() as usize);
    histfile.read_to_string(&mut buf)?;
    println!("{buf}");
    Ok(())
}
