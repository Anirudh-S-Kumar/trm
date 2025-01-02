use crate::LOG_FILE;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Error, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum OpType {
    TRASH,
    RESTORE,
}

impl OpType {
    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        match self {
            OpType::TRASH => "Trash",
            OpType::RESTORE => "Restore",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// The original path from where the path was moved
    pub src: Vec<String>,

    /// The path where the file ended up in  
    pub dst: Vec<String>,

    /// Type of operation
    pub operation: OpType,

    /// The datetime when it was moved
    pub moved_time: DateTime<Local>,
}

pub fn append_to_logs(info: &FileInfo) -> Result<(), Error> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(LOG_FILE)
        .unwrap();

    let mut writer = io::BufWriter::new(file);
    let serialized_info = serde_json::to_string(info)?;
    writeln!(writer, "{}", serialized_info)?;
    writer.flush()?;
    Ok(())
}


pub fn _read_logs() -> io::Result<Vec<FileInfo>> {
    let file = File::open(LOG_FILE)?;
    let reader = BufReader::new(file);
    let mut logs: Vec<FileInfo> = vec![];

    for line in reader.lines() {
        let line = line?;
        if let Ok(log) = serde_json::from_str::<FileInfo>(&line) {
            logs.push(log);
        }
    }

    Ok(logs)
}
