use anyhow::{Context, Result, bail};
use reverse_lines::ReverseLines;
use std::{
    fmt,
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::PathBuf,
};

fn init_histfile(histfile: &File) -> Result<()> {
    todo!()
}

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
    hist_type: HistoryType,
    paths: Vec<PathBuf>,
}

pub fn make_histfile_entry(
    mut histfile: File,
    hist_type: HistoryType,
    paths: Vec<PathBuf>,
) -> Result<()> {
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

fn read_histfile_entry(histfile: File, _index: usize) -> Result<Option<HistfileEntry>> {
    let rev_lines = ReverseLines::new(BufReader::new(histfile))
        .context("Could not iterate through histfile lines")?;

    let last_line = match rev_lines.into_iter().nth(0) {
        Some(line) => line,
        None => return Ok(None),
    }
    .context("Could not get nth line from histfile")?;

    let words: Vec<&str> = last_line.split("\0").into_iter().collect();

    // SAFETY: assert
    assert!(words.len() >= 2);
    let hist_type = match *(unsafe { words.get_unchecked(0) }) {
        "mv" => HistoryType::Move,
        "cp" => HistoryType::Copy,
        _ => bail!("Invalid history type"),
    };

    let paths: Vec<PathBuf> = words[1..].iter().copied().map(PathBuf::from).collect();

    Ok(Some(HistfileEntry { hist_type, paths }))
}

pub fn histfile_contents(mut histfile: File) -> Result<()> {
    let mut buf = String::with_capacity(histfile.metadata().unwrap().len() as usize);
    histfile.read_to_string(&mut buf)?;
    println!("{buf}");
    Ok(())
}

pub fn paste(histfile: File, dest: Option<PathBuf>) -> Result<()> {
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

    if let Some(entry) = read_histfile_entry(histfile, 0)? {
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
            HistoryType::Move => todo!(),
        };
    };

    Ok(())
}
