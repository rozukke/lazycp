use anyhow::{Context, Result};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
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

pub fn histfile_entry(
    mut histfile: File,
    hist_type: HistoryType,
    paths: Vec<PathBuf>,
) -> Result<()> {
    writeln!(
        histfile,
        "'{hist_type}' '{}'",
        paths
            .iter()
            .filter_map(|x| x.to_str())
            .collect::<Vec<&str>>()
            .join("' '")
    )
    .context("Could not create entry in histfile. Aborting...")?;
    Ok(())
}

pub fn histfile_contents(mut histfile: File) -> Result<()> {
    let mut buf = String::with_capacity(histfile.metadata().unwrap().len() as usize);
    histfile.read_to_string(&mut buf)?;
    println!("{buf}");
    Ok(())
}

