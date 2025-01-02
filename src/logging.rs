use crate::LOG_FILE;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use slog::{info, o, Drain, Logger};
use std::{
    fs::{File, OpenOptions},
    io,
    io::{BufRead, BufReader},
    sync::Mutex,
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
    pub src: String,

    /// The path where the file ended up in  
    pub dst: String,

    /// Type of operation
    pub operation: OpType,

    /// The datetime when it was moved
    pub moved_time: DateTime<Local>,
}

fn init_logger() -> Logger {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(LOG_FILE)
        .unwrap();

    let drain = slog_json::Json::new(file).build().fuse();

    Logger::root(Mutex::new(drain).fuse(), o!())
}

pub fn append_to_logs(info: &FileInfo) {
    let logger = init_logger();
    info!(logger, "File Operation";
        "SRC" => &info.src,
        "DST" => &info.dst,
        "OPERATION" => &info.operation.as_str(),
        "MOVED_TIME" => &info.moved_time.to_string()
    );
}

pub fn _read_logs() -> io::Result<Vec<FileInfo>> {
    let file = File::open(LOG_FILE).unwrap();
    let reader = BufReader::new(file);
    let mut logs: Vec<FileInfo> = vec![];

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(log) = serde_json::from_str::<FileInfo>(&line) {
            logs.push(log);
        }
    }

    Ok(logs)
}
